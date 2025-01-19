use std::{
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::Result;
use common::name::TimedName;

use crate::{
    cloudlet::driver::process::{drop_process, kill_process, try_wait},
    driver::template::Template,
    error, info, warn,
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
    pub folder: PathBuf,
    pub template: Rc<Template>,
}

impl LocalUnit {
    pub fn new(name: &TimedName, folder: &Path, template: Rc<Template>) -> Self {
        Self {
            state: UnitState::Stopped,
            changed: Instant::now(),
            pid: None,
            name: name.clone(),
            folder: folder.to_path_buf(),
            template,
        }
    }

    pub fn tick(&mut self) -> bool {
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
                false
            }
            _ => true,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.pid = Some(self.template.run_startup(&self.folder)?);
        self.state = UnitState::Running;
        self.changed = Instant::now();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(pid) = self.pid {
            self.template.run_shutdown(pid)?;
        }
        self.state = UnitState::Stopping;
        self.changed = Instant::now();
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
