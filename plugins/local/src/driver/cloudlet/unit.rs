use std::{path::PathBuf, rc::Rc, time::Instant};

use anyhow::{anyhow, Result};
use common::{name::TimedName, tick::TickResult};

use crate::{
    node::plugin::{
        file::remove_dir_all,
        process::{drop_process, kill_process, read_line_async, try_wait, StdReader},
        types::{Directory, KeyValue},
    },
    plugin::{config::UNIT_STOP_TIMEOUT, template::Template, LocalCloudletWrapper},
    exports::node::plugin::bridge::{Retention, Unit},
    info,
    storage::Storage,
    warn,
};

/* Variables */
const CONTROLLER_ADDRESS: &str = "CONTROLLER_ADDRESS";
const UNIT_TOKEN: &str = "UNIT_TOKEN";
const UNIT_PORT: &str = "UNIT_PORT";

#[derive(PartialEq)]
pub enum UnitState {
    Running,
    Restarting,
    Stopping,
    Stopped,
}

pub struct LocalUnit {
    pub server: Unit,
    pub state: UnitState,
    pub changed: Instant,
    pub pid: Option<u32>,
    pub name: TimedName,
    pub _internal_folder: PathBuf,
    pub host_folder: Directory,
    pub template: Rc<Template>,
}

impl LocalUnit {
    pub fn new(
        node: &LocalCloudletWrapper,
        mut server: Unit,
        name: &TimedName,
        template: Rc<Template>,
    ) -> Self {
        let environment = &mut server.allocation.spec.environment;
        environment.push(KeyValue {
            key: CONTROLLER_ADDRESS.to_string(),
            value: node.inner.controller.address.clone(),
        });
        environment.push(KeyValue {
            key: UNIT_TOKEN.to_string(),
            value: server.auth.token.clone(),
        });
        environment.push(KeyValue {
            key: UNIT_PORT.to_string(),
            value: server.allocation.addresses[0].port.to_string(),
        });
        Self {
            state: UnitState::Stopped,
            changed: Instant::now(),
            pid: None,
            _internal_folder: Storage::get_server_folder(name, &server.allocation.spec.disk_retention),
            host_folder: Storage::get_server_folder_host_converted(
                name,
                &server.allocation.spec.disk_retention,
            ),
            server,
            name: name.clone(),
            template,
        }
    }

    pub fn tick(&mut self) -> Result<TickResult> {
        if let Some(pid) = self.pid {
            while let Some(line) = read_line_async(pid, StdReader::Stdout).ok().flatten() {
                info!("<blue>[{}]</> {}", self.name.get_raw_name(), line.trim());
            }
        }

        match self.state {
            UnitState::Restarting | UnitState::Stopping => {
                let pid = match self.pid {
                    Some(pid) => pid,
                    None => {
                        self.state = UnitState::Stopped;
                        return Ok(TickResult::Drop);
                    }
                };

                if self.changed.elapsed() > UNIT_STOP_TIMEOUT {
                    warn!(
                        "Unit {} failed to stop in time, killing it",
                        self.name.get_raw_name()
                    );
                    kill_process(pid).map_err(|error| anyhow!(error))?;
                } else {
                    match try_wait(pid) {
                        Ok(Some(code)) => {
                            info!(
                                "Unit <blue>{}</> <red>stopped</> with exit code <red>{}</>",
                                self.name.get_raw_name(),
                                code
                            );
                            drop_process(pid).map_err(|error| anyhow!(error))?;
                        }
                        Ok(None) => return Ok(TickResult::Ok),
                        Err(error) => {
                            warn!("Failed to get status for server {}, killing it", error);
                            kill_process(pid).map_err(|error| anyhow!(error))?;
                        }
                    }
                }

                self.pid = None;
                self.changed = Instant::now();
                if self.state == UnitState::Restarting {
                    self.start()?;
                } else {
                    self.state = UnitState::Stopped;
                }
                self.state = UnitState::Stopped;
                Ok(TickResult::Ok)
            }
            UnitState::Stopped => {
                self.cleanup()?;
                Ok(TickResult::Drop)
            }
            _ => Ok(TickResult::Ok),
        }
    }

    fn cleanup(&self) -> Result<()> {
        if self.server.allocation.spec.disk_retention == Retention::Temporary {
            remove_dir_all(&self.host_folder).map_err(|error| anyhow!(error))?;
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        self.pid = Some(
            self.template
                .run_startup(&self.host_folder, &self.server.allocation.spec.environment)?,
        );
        self.state = UnitState::Running;
        self.changed = Instant::now();
        Ok(())
    }

    pub fn kill(&mut self) -> Result<()> {
        if let Some(pid) = self.pid {
            kill_process(pid).map_err(|error| anyhow!(error))?;
            self.state = UnitState::Stopped;
            self.changed = Instant::now();
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(pid) = self.pid {
            self.template.run_shutdown(pid)?;
            self.state = UnitState::Stopping;
            self.changed = Instant::now();
        }
        Ok(())
    }

    pub fn restart(&mut self) -> Result<()> {
        if let Some(pid) = self.pid {
            self.template.run_shutdown(pid)?;
        }
        self.state = UnitState::Restarting;
        self.changed = Instant::now();
        Ok(())
    }
}
