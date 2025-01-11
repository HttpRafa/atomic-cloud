use std::process::Command;

use super::{generated::cloudlet::driver, WasmDriverState};

impl driver::child_process::Host for WasmDriverState {
    fn spawn_child_process(&mut self, command: String, args: Vec<String>) -> Result<u32, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let command = Command::new(command).args(args).spawn();
        match command {
            Ok(child) => {
                let pid = child.id();
                driver
                    .data
                    .child_processes
                    .write()
                    .map_err(|_| "Failed to acquire write lock on child processes")?
                    .insert(pid, child);
                Ok(pid)
            }
            Err(error) => Err(format!("Failed to spawn child process: {}", error)),
        }
    }

    fn kill_child_process(&mut self, pid: u32) -> Result<bool, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut child_processes = driver
            .data
            .child_processes
            .write()
            .map_err(|_| "Failed to acquire write lock on child processes")?;

        if let Some(mut child) = child_processes.remove(&pid) {
            child
                .kill()
                .map_err(|error| format!("Failed to kill child process: {}", error))
                .map(|_| true)
        } else {
            Ok(false)
        }
    }
}
