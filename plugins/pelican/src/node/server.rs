use std::time::Instant;

use anyhow::{bail, Result};
use common::name::TimedName;

use crate::{
    error,
    generated::{
        exports::plugin::system::{
            bridge::{self, DiskRetention, Guard},
            screen::ScreenType,
        },
        plugin::system::tls::get_certificate,
    },
    plugin::config::Config,
    warn,
};

use super::{
    backend::{
        allocation::data::BAllocation,
        server::data::{BServer, BServerEgg, BServerFeatureLimits},
    },
    InnerNode,
};

pub mod manager;

/* Variables */
const CONTROLLER_ADDRESS: &str = "CONTROLLER_ADDRESS";
const CONTROLLER_CERTIFICATE: &str = "CONTROLLER_CERTIFICATE";
const SERVER_TOKEN: &str = "SERVER_TOKEN";
const SERVER_PORT: &str = "SERVER_PORT";

pub struct Server {
    name: TimedName,
    request: bridge::Server,
    egg: BServerEgg,

    backend: (u32, String),

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
                state: State::Running,
                guard: None,
            })
        } else {
            bail!("Failed to create server on the panel");
        }
    }

    #[allow(dead_code)]
    pub fn cleanup(&mut self, node: &str) -> Result<()> {
        Ok(())
    }

    pub fn tick(&mut self, config: &Config) -> Result<&State> {
        Ok(&self.state)
    }

    pub fn screen(&self) -> ScreenType {
        ScreenType::Unsupported
    }
}

pub enum State {
    Running,
    Restarting(Instant),
    Stopping(Instant),
    Dead,
}
