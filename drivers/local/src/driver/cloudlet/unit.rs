use std::{
    fs,
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use common::name::TimedName;

use crate::{
    cloudlet::driver::process::{drop_process, kill_process, read_line, try_wait, StdReader}, debug, driver::template::Template, error, exports::cloudlet::driver::bridge::Retention, info, storage::Storage, warn
};

const STOP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(PartialEq)]
pub enum UnitState {
    Running,
    Restarting,
    Stopping,
    Stopped,
}

pub struct LocalUnit {
    pub state: UnitState,
    pub changed: Instant,
    pub pid: Option<u32>,
    pub name: TimedName,
    pub internal_folder: PathBuf,
    pub child_folder: PathBuf,
    pub retention: Retention,
    pub template: Rc<Template>,
}

impl LocalUnit {
    pub fn new(name: &TimedName, retention: &Retention, template: Rc<Template>) -> Self {
        Self {
            state: UnitState::Stopped,
            changed: Instant::now(),
            pid: None,
            name: name.clone(),
            internal_folder: Storage::get_unit_folder(name, retention),
            child_folder: Storage::get_unit_folder_outside(name, retention),
            retention: *retention,
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
                            debug!("<blue>[{}]</> {}", self.name.get_raw_name(), line);
                        }
                    }
                    Err(error) => {
                        last_size = 0;
                        debug!(
                            "Failed to read stdout of process <blue>{}</>: <red>{}</>",
                            pid, error
                        );
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
        if self.retention == Retention::Temporary {
            if let Err(error) = fs::remove_dir_all(&self.internal_folder) {
                error!(
                    "Failed to remove folder for unit {}: {}",
                    self.name.get_raw_name(),
                    error
                );
            }
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.pid = Some(self.template.run_startup(&self.child_folder)?);
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
            if self.retention == Retention::Temporary {
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
