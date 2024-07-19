use std::process::{exit, Child, Command, Stdio};

use colored::Colorize;
use log::{error, info};

pub struct ManagedProcess {
    /* Process */
    process: Child,
}

impl ManagedProcess {
    pub fn start(program: &str, args: &[String]) -> Self {
        info!("{} child process...", "Starting".green());
        info!("-> {} {}", program.blue(), args.join(" "));
        Self {
            process: match Command::new(&program)
                .args(args)
                .stdin(Stdio::inherit())
                .spawn()
            {
                Ok(child) => child,
                Err(error) => {
                    error!("{} to start child process: {}", "Failed".red(), error);
                    exit(1);
                }
            },
        }
    }

    pub fn tick(&mut self) -> bool {
        if let Some(status) = self.process.try_wait().ok().flatten() {
            info!(
                "Child process {} with {}",
                "exited".red(),
                format!("{}", status).blue()
            );
            return true;
        }
        false
    }

    pub fn kill_if_running(&mut self) {
        if self.process.try_wait().ok().flatten().is_none() {
            info!("{} child process...", "Stopping".red());
            self.process.kill().expect("Failed to kill child process");
            self.process
                .wait()
                .expect("Failed to wait for child process");
        }
    }
}
