use std::{
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use common::name::TimedName;

use crate::{
    cloudlet::driver::{
        file::{remove_dir_all, Directory},
        process::{drop_process, kill_process, read_line, try_wait, StdReader},
        types::{KeyValue, Reference},
    },
    driver::{template::Template, LocalCloudletWrapper},
    error,
    exports::cloudlet::driver::bridge::{Retention, Unit},
    info,
    storage::Storage,
    warn,
};

/* Timeouts */
const STOP_TIMEOUT: Duration = Duration::from_secs(30);

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
    pub unit: Unit,
    pub state: UnitState,
    pub changed: Instant,
    pub pid: Option<u32>,
    pub name: TimedName,
    pub _internal_folder: PathBuf,
    pub child_folder: PathBuf,
    pub template: Rc<Template>,
}

impl LocalUnit {
    pub fn new(
        node: &LocalCloudletWrapper,
        mut unit: Unit,
        name: &TimedName,
        template: Rc<Template>,
    ) -> Self {
        let environment = &mut unit.allocation.spec.environment;
        environment.push(KeyValue {
            key: CONTROLLER_ADDRESS.to_string(),
            value: node.inner.controller.address.clone(),
        });
        environment.push(KeyValue {
            key: UNIT_TOKEN.to_string(),
            value: unit.auth.token.clone(),
        });
        environment.push(KeyValue {
            key: UNIT_PORT.to_string(),
            value: unit.allocation.addresses[0].port.to_string(),
        });
        Self {
            state: UnitState::Stopped,
            changed: Instant::now(),
            pid: None,
            _internal_folder: Storage::get_unit_folder(name, &unit.allocation.spec.disk_retention),
            child_folder: Storage::get_unit_folder_outside(
                name,
                &unit.allocation.spec.disk_retention,
            ),
            unit,
            name: name.clone(),
            template,
        }
    }

    pub fn tick(&mut self) -> bool {
        // TODO: Remove this if we have a system that can handle screens
        if let Some(pid) = self.pid {
            let mut last_size = 10000;
            while last_size > 0 {
                match read_line(pid, StdReader::Stdout) {
                    Ok(read) => {
                        last_size = read.0;
                        if read.0 > 0 {
                            let line = read.1.trim();
                            info!("<blue>[{}]</> {}", self.name.get_raw_name(), line);
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        }

        match self.state {
            UnitState::Restarting | UnitState::Stopping => {
                let pid = self
                    .pid
                    .expect("Unit is marked as restarting/stopping without a PID");

                if self.changed.elapsed() > STOP_TIMEOUT {
                    warn!(
                        "Failed to stop unit {} in time, killing it",
                        self.name.get_raw_name()
                    );
                    if let Err(error) = kill_process(pid) {
                        error!(
                            "Failed to kill unit {}: {}",
                            self.name.get_raw_name(),
                            error
                        );
                    }
                } else {
                    match try_wait(pid) {
                        Ok(Some(code)) => {
                            info!(
                                "Unit {} stopped with exit code {}",
                                self.name.get_raw_name(),
                                code
                            );
                            if let Err(error) = drop_process(pid) {
                                error!(
                                    "Failed to drop unit {}: {}",
                                    self.name.get_raw_name(),
                                    error
                                );
                            }
                        }
                        Ok(None) => return true,
                        Err(error) => {
                            warn!("Failed to get status for unit {}, killing it", error);
                            if let Err(error) = kill_process(pid) {
                                error!(
                                    "Failed to kill unit {}: {}",
                                    self.name.get_raw_name(),
                                    error
                                );
                            }
                        }
                    }
                }

                self.pid = None;
                self.changed = Instant::now();
                self.state = UnitState::Stopped;

                if self.state == UnitState::Restarting {
                    if let Err(error) = self.start() {
                        warn!("Failed to restart unit: {}", error);
                    }
                    return true;
                }
                self.cleanup();
                false
            }
            UnitState::Stopped => {
                self.cleanup();
                false
            }
            _ => true,
        }
    }

    fn cleanup(&self) {
        if self.unit.allocation.spec.disk_retention == Retention::Temporary {
            if let Err(error) = remove_dir_all(&Directory {
                path: self.child_folder.to_string_lossy().to_string(),
                reference: Reference::Data,
            }) {
                error!(
                    "Failed to remove folder for unit {}: {}",
                    self.name.get_raw_name(),
                    error
                );
            }
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.pid = Some(self.template.run_startup(
            &self.child_folder,
            self.unit.allocation.spec.environment.clone(),
        )?);
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
            if self.unit.allocation.spec.disk_retention == Retention::Temporary {
                self.kill()?;
            } else {
                self.template.run_shutdown(pid)?;
                self.state = UnitState::Stopping;
                self.changed = Instant::now();
            }
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
