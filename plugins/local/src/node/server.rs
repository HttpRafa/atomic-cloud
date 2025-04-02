use std::{fmt::Display, rc::Rc, time::Instant};

use anyhow::{anyhow, Result};
use common::name::TimedName;

use crate::{
    debug,
    generated::{
        exports::plugin::system::{
            bridge::{self, Guard},
            screen::{Screen as GenericScreen, ScreenType},
        },
        plugin::system::{
            data_types::DiskRetention,
            file::remove_dir_all,
            process::{ExitStatus, Process, ProcessBuilder},
            tls::get_certificate,
        },
    },
    info,
    plugin::config::Config,
    storage::Storage,
    template::Template,
    warn,
};

use super::{screen::Screen, InnerNode};

pub mod manager;

/* Variables */
const CONTROLLER_ADDRESS: &str = "CONTROLLER_ADDRESS";
const CONTROLLER_CERTIFICATE: &str = "CONTROLLER_CERTIFICATE";
const SERVER_TOKEN: &str = "SERVER_TOKEN";
const SERVER_MEMORY: &str = "SERVER_MEMORY";
const SERVER_PORT: &str = "SERVER_PORT";

pub struct Server {
    name: TimedName,
    request: bridge::Server,
    template: String,
    builder: ProcessBuilder,
    process: Rc<Process>,

    state: State,
    guard: Option<Guard>,
}

impl Server {
    pub fn spawn(node: &InnerNode, request: bridge::Server, template: &Template) -> Result<Self> {
        let name = TimedName::new_no_identifier(
            &request.name,
            matches!(
                request.allocation.spec.disk_retention,
                DiskRetention::Permanent
            ),
        );

        // Prepare the directory
        {
            let directory = Storage::server_directory(
                false,
                &node.name,
                &name,
                &request.allocation.spec.disk_retention,
            );
            if !directory.exists() {
                template.copy_self(&directory)?;
            }
        }

        // Prepare the environment
        let mut environment = request.allocation.spec.environment.clone();
        environment.reserve(4);
        environment.push((CONTROLLER_ADDRESS.to_string(), node.controller.clone()));
        environment.push((SERVER_TOKEN.to_string(), request.token.clone()));
        environment.push((
            SERVER_MEMORY.to_string(),
            request.allocation.resources.memory.to_string(),
        ));
        environment.push((
            SERVER_PORT.to_string(),
            request
                .allocation
                .ports
                .first()
                .ok_or(anyhow!("No ports allocated"))?
                .port
                .to_string(),
        ));

        // If we use a certificate, add it to the environment
        if let Some(certificate) = get_certificate() {
            environment.push((CONTROLLER_CERTIFICATE.to_string(), certificate));
        }

        // Spawn the server
        let (process, builder) = template.spawn(
            environment,
            &Storage::create_server_directory(
                &node.name,
                &name,
                &request.allocation.spec.disk_retention,
            ),
        )?;

        Ok(Self {
            name,
            request,
            template: template.name().to_string(),
            builder,
            process: Rc::new(process),
            state: State::Running,
            guard: None,
        })
    }

    pub fn cleanup(&mut self, node: &str) -> Result<()> {
        debug!("Cleaning up server {}", self.name.get_name());
        if matches!(
            self.request.allocation.spec.disk_retention,
            DiskRetention::Temporary
        ) {
            remove_dir_all(&Storage::create_server_directory(
                node,
                &self.name,
                &DiskRetention::Temporary,
            ))
            .map_err(|error| anyhow!(error))?;
        }
        debug!("Cleaned up server {}", self.name.get_name());

        Ok(())
    }

    pub fn tick(&mut self, config: &Config) -> Result<&State> {
        self.state = match self.state {
            State::Restarting(instant) => {
                if &instant.elapsed() > config.restart_timeout() {
                    warn!(
                        "Server {} failed to stop in {:?}. Killing and respawning process...",
                        self.name.get_name(),
                        config.restart_timeout()
                    );
                    self.respawn()?;
                    State::Running
                } else if let Some(code) =
                    self.process.try_wait().map_err(|error| anyhow!(error))?
                {
                    info!(
                        "Server {} exited with status {}",
                        self.name.get_name(),
                        code
                    );
                    self.respawn()?;
                    State::Running
                } else {
                    State::Restarting(instant)
                }
            }
            State::Stopping(instant) => {
                if &instant.elapsed() > config.stop_timeout() {
                    warn!(
                        "Server {} failed to stop in {:?}. Killing process...",
                        self.name.get_name(),
                        config.stop_timeout()
                    );
                    if let Err(error) = self.kill() {
                        warn!("Failed to kill process: {}", error);
                    }
                    State::Dead
                } else if let Some(code) =
                    self.process.try_wait().map_err(|error| anyhow!(error))?
                {
                    info!(
                        "Server {} exited with status {}",
                        self.name.get_name(),
                        code
                    );
                    State::Dead
                } else {
                    State::Stopping(instant)
                }
            }
            _ => State::Running,
        };
        Ok(&self.state)
    }

    pub fn restart(&mut self, node: &InnerNode) -> Result<()> {
        let templates = node.templates.borrow();
        let template = templates
            .get_template(&self.template)
            .ok_or(anyhow!("Template not found while restarting server"))?;

        self.state = State::Restarting(Instant::now());
        template.write_shutdown(&self.process)?;
        Ok(())
    }

    pub fn stop(&mut self, node: &InnerNode, guard: Guard) -> Result<()> {
        self.state = State::Stopping(Instant::now());
        self.guard = Some(guard);
        let templates = node.templates.borrow();
        let template = templates
            .get_template(&self.template)
            .ok_or(anyhow!("Template not found while stopping server"))?;
        template.write_shutdown(&self.process)?;
        Ok(())
    }

    fn kill(&mut self) -> Result<()> {
        self.process.kill().map_err(|error| anyhow!(error))?;
        Ok(())
    }

    fn respawn(&mut self) -> Result<()> {
        if let Err(error) = self.kill() {
            warn!("Failed to kill process: {}", error);
        }
        self.process = Rc::new(self.builder.spawn().map_err(|error| anyhow!(error))?);
        Ok(())
    }

    pub fn screen(&self) -> bridge::ScreenType {
        ScreenType::Supported(GenericScreen::new(Screen(Rc::clone(&self.process))))
    }
}

pub enum State {
    Running,
    Restarting(Instant),
    Stopping(Instant),
    Dead,
}

impl Display for ExitStatus {
    fn fmt(&self, format: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitStatus::Code(code) => write!(format, "code {}", code),
            ExitStatus::Signal(signal) => write!(format, "signal {}", signal),
            ExitStatus::Unknown => write!(format, "unknown"),
        }
    }
}
