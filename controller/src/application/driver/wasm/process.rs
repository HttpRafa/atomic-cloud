use std::{
    io::{Read, Write},
    process::Command,
};

use super::{generated::cloudlet::driver, WasmDriverState};

impl driver::process::Host for WasmDriverState {
    fn spawn_process(&mut self, command: String, args: Vec<String>) -> Result<u32, String> {
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

    fn kill_process(&mut self, pid: u32) -> Result<bool, String> {
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

    fn read_stdout(&mut self, pid: u32) -> Result<Vec<u8>, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut child_processes = driver
            .data
            .child_processes
            .write()
            .map_err(|_| "Failed to acquire write lock on child processes")?;

        if let Some(child) = child_processes.get_mut(&pid) {
            let output = child
                .stdout
                .as_mut()
                .ok_or("Failed to open stdout of child process")?;
            let mut buffer = Vec::new();
            output
                .read_to_end(&mut buffer)
                .map_err(|error| format!("Failed to read stdout of child process: {}", error))?;
            Ok(buffer)
        } else {
            Err("Child process does not exist".to_string())
        }
    }

    fn read_stderr(&mut self, pid: u32) -> Result<Vec<u8>, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut child_processes = driver
            .data
            .child_processes
            .write()
            .map_err(|_| "Failed to acquire write lock on child processes")?;

        if let Some(child) = child_processes.get_mut(&pid) {
            let output = child
                .stderr
                .as_mut()
                .ok_or("Failed to open stderr of child process")?;
            let mut buffer = Vec::new();
            output
                .read_to_end(&mut buffer)
                .map_err(|error| format!("Failed to read stderr of child process: {}", error))?;
            Ok(buffer)
        } else {
            Err("Child process does not exist".to_string())
        }
    }

    fn write_stdin(&mut self, pid: u32, data: Vec<u8>) -> Result<bool, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut child_processes = driver
            .data
            .child_processes
            .write()
            .map_err(|_| "Failed to acquire write lock on child processes")?;

        if let Some(child) = child_processes.get_mut(&pid) {
            let input = child
                .stdin
                .as_mut()
                .ok_or("Failed to open stdin of child process")?;
            input
                .write_all(&data)
                .map_err(|error| format!("Failed to write to stdin of child process: {}", error))?;
            Ok(true)
        } else {
            Err("Child process does not exist".to_string())
        }
    }
}