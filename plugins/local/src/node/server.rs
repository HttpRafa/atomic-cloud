use std::{rc::Rc, time::Instant};

use anyhow::{anyhow, Result};
use common::name::TimedName;

use crate::{
    generated::{
        exports::plugin::system::{
            bridge::{self, DiskRetention},
            screen::{GenericScreen, ScreenType},
        },
        plugin::system::process::{Process, ProcessBuilder},
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
const SERVER_TOKEN: &str = "SERVER_TOKEN";
const SERVER_PORT: &str = "SERVER_PORT";

pub struct Server {
    name: TimedName,
    request: bridge::Server,
    template: String,
    builder: ProcessBuilder,
    process: Rc<Process>,

    state: State,
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
        environment.reserve(3);
        environment.push((CONTROLLER_ADDRESS.to_string(), node.controller.clone()));
        environment.push((SERVER_TOKEN.to_string(), request.token.clone()));
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
        })
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
                    info!("Server {} exited with code {}", self.name.get_name(), code);
                    self.respawn()?;
                    State::Running
                } else {
                    State::Restarting(instant)
                }
            }
            State::Stopping(instant) => {
                if &instant.elapsed() > config.restart_timeout() {
                    warn!(
                        "Server {} failed to stop in {:?}. Killing process...",
                        self.name.get_name(),
                        config.restart_timeout()
                    );
                    self.kill()?;
                    State::Dead
                } else if let Some(code) =
                    self.process.try_wait().map_err(|error| anyhow!(error))?
                {
                    info!("Server {} exited with code {}", self.name.get_name(), code);
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

    pub fn stop(&mut self, node: &InnerNode) -> Result<()> {
        self.state = State::Stopping(Instant::now());
        match self.request.allocation.spec.disk_retention {
            DiskRetention::Temporary => {
                self.process
                    .kill()
                    .map_err(|error| anyhow!("Failed to kill process: {}", error))?;
            }
            DiskRetention::Permanent => {
                let templates = node.templates.borrow();
                let template = templates
                    .get_template(&self.template)
                    .ok_or(anyhow!("Template not found while stopping server"))?;
                template.write_shutdown(&self.process)?;
            }
        }
        Ok(())
    }

    fn kill(&mut self) -> Result<()> {
        self.process.kill().map_err(|error| anyhow!(error))?;
        Ok(())
    }

    fn respawn(&mut self) -> Result<()> {
        self.kill()?;
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
