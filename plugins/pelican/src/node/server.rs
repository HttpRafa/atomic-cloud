use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use anyhow::{bail, Result};
use common::name::TimedName;
use url::Url;

use crate::{
    debug, error,
    generated::{
        exports::plugin::system::{
            bridge::{self, DiskRetention, Guard},
            screen::{Screen as GenericScreen, ScreenType},
        },
        plugin::system::tls::get_certificate,
    },
    info,
    plugin::config::Config,
    warn,
};

use super::{
    backend::{
        allocation::data::BAllocation,
        server::data::{BServer, BServerEgg, BServerFeatureLimits, PanelState},
    },
    screen::Screen,
    InnerNode,
};

pub mod manager;

/* Variables */
const CONTROLLER_ADDRESS: &str = "CONTROLLER_ADDRESS";
const CONTROLLER_CERTIFICATE: &str = "CONTROLLER_CERTIFICATE";
const SERVER_TOKEN: &str = "SERVER_TOKEN";

/* Durations */
const DEFAULT_UPDATE_DURATION: Duration = Duration::from_secs(15);

pub struct Server {
    name: TimedName,
    #[allow(unused)]
    request: bridge::Server,
    #[allow(unused)]
    egg: BServerEgg,

    backend: (u32, String),

    last_update: Instant,

    state: State,
    guard: Option<Guard>,
}

impl Server {
    pub fn start(node: &InnerNode, request: bridge::Server) -> Result<Self> {
        let name = TimedName::new(
            &node.identifier,
            &request.name,
            matches!(
                request.allocation.spec.disk_retention,
                DiskRetention::Permanent
            ),
        );

        // Get backend allocations
        let allocations = node
            .allocations
            .borrow()
            .get_allocations(&request.allocation.ports);
        if allocations.len() != request.allocation.ports.len() {
            warn!("The allocation manager failed to map some ports of the server {} to there backend variant. This may cause issues.", request.name);
        }

        // Build egg from request
        let egg = {
            let mut id = None;
            let mut startup = None;
            for value in &request.allocation.spec.settings {
                match value.0.as_str() {
                    "egg" => match value.1.parse::<u32>() {
                        Ok(value) => {
                            id = Some(value);
                        }
                        Err(_) => {
                            error!("The egg setting must be a number!");
                        }
                    },
                    "startup" => {
                        startup = Some(value.1.clone());
                    }
                    _ => {}
                }
            }

            if id.is_none() {
                bail!("The following required settings to start the server are missing: egg");
            }
            BServerEgg {
                id: id.unwrap(),
                startup,
            }
        };

        // Prepare the environment
        let mut environment = request.allocation.spec.environment.clone();
        environment.reserve(4);
        environment.push((CONTROLLER_ADDRESS.to_string(), node.controller.clone()));
        environment.push((SERVER_TOKEN.to_string(), request.token.clone()));

        // If we use a certificate, add it to the environment
        if let Some(certificate) = get_certificate() {
            environment.push((CONTROLLER_CERTIFICATE.to_string(), certificate));
        }

        if let Some(server) = node.backend.get_server_by_name(&name) {
            Self::update(node, request, server, name, allocations, egg, &environment)
        } else {
            Self::create(node, request, name, allocations, egg, &environment)
        }
    }

    fn update(
        node: &InnerNode,
        request: bridge::Server,
        server: BServer,
        name: TimedName,
        allocations: Vec<BAllocation>,
        egg: BServerEgg,
        environment: &[(String, String)],
    ) -> Result<Self> {
        if matches!(
            request.allocation.spec.disk_retention,
            DiskRetention::Temporary
        ) {
            bail!("The server {} already exists on the panel, but the disk retention is set to temporary. How is that possible?", name.get_name());
        }

        node.backend
            .update_settings(&server, &request, allocations[0].id, environment);
        node.backend.start_server(&server.identifier);
        Ok(Self {
            name,
            request,
            egg,
            backend: (server.id, server.identifier),
            last_update: Instant::now(),
            state: State::Running,
            guard: None,
        })
    }

    fn create(
        node: &InnerNode,
        request: bridge::Server,
        name: TimedName,
        allocations: Vec<BAllocation>,
        egg: BServerEgg,
        environment: &[(String, String)],
    ) -> Result<Self> {
        if let Some(server) = node.backend.create_server(
            &name,
            &request,
            &allocations,
            environment,
            &egg,
            &BServerFeatureLimits {
                databases: 0,
                backups: 0,
            },
        ) {
            Ok(Self {
                name,
                request,
                egg,
                backend: (server.id, server.identifier),
                last_update: Instant::now(),
                state: State::Running,
                guard: None,
            })
        } else {
            bail!("Failed to create server on the panel");
        }
    }

    // Currently unused
    pub fn cleanup(&mut self, node: &InnerNode) -> Result<()> {
        debug!("Cleaning up server {}", self.name.get_name());
        if matches!(
            self.request.allocation.spec.disk_retention,
            DiskRetention::Temporary
        ) {
            node.backend.delete_server(self.backend.0);
        }
        debug!("Cleaned up server {}", self.name.get_name());
        Ok(())
    }

    pub fn tick(&mut self, node: &InnerNode, config: &Config) -> Result<&State> {
        self.state = match self.state {
            State::Restarting(instant) => 'update: {
                if &instant.elapsed() > config.restart_timeout() {
                    warn!(
                        "Server {} failed to stop in {:?}. Killing and respawning process...",
                        self.name.get_name(),
                        config.restart_timeout()
                    );
                    node.backend.restart_server(&self.backend.1);
                    break 'update State::Running;
                } else if self.last_update.elapsed() >= DEFAULT_UPDATE_DURATION
                    && let Some(state) = node.backend.get_server_state(&self.backend.1)
                {
                    if matches!(state, PanelState::Running) {
                        info!("Server {} is now running again.", self.name.get_name(),);
                        break 'update State::Running;
                    } else {
                        self.last_update = Instant::now();
                    }
                }
                State::Restarting(instant)
            }
            State::Stopping(instant) => 'update: {
                if &instant.elapsed() > config.stop_timeout() {
                    warn!(
                        "Server {} failed to stop in {:?}. Killing process...",
                        self.name.get_name(),
                        config.stop_timeout()
                    );
                    node.backend.kill_server(&self.backend.1);
                    break 'update State::Dead;
                } else if self.last_update.elapsed() >= DEFAULT_UPDATE_DURATION
                    && let Some(state) = node.backend.get_server_state(&self.backend.1)
                {
                    if matches!(state, PanelState::Offline) {
                        info!("Server {} stopped successfully.", self.name.get_name(),);
                        break 'update State::Dead;
                    } else {
                        self.last_update = Instant::now();
                    }
                }
                State::Stopping(instant)
            }
            _ => State::Running,
        };
        Ok(&self.state)
    }

    pub fn restart(&mut self, node: &InnerNode) -> Result<()> {
        self.state = State::Restarting(Instant::now());
        node.backend.restart_server(&self.backend.1);
        Ok(())
    }

    pub fn stop(&mut self, node: &InnerNode, guard: Guard) -> Result<()> {
        self.state = State::Stopping(Instant::now());
        self.guard = Some(guard);
        node.backend.stop_server(&self.backend.1);
        Ok(())
    }

    pub fn screen(&self, url: &Url) -> ScreenType {
        let console_url = url
            .join(&format!("server/{}/console", self.backend.0))
            .expect("Failed to join URL");
        ScreenType::Supported(GenericScreen::new(Screen(
            console_url.to_string(),
            RefCell::new(true),
        )))
    }
}

pub enum State {
    Running,
    Restarting(Instant),
    Stopping(Instant),
    Dead,
}
