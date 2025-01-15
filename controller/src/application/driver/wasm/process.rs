use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::storage::Storage;

use super::{
    generated::cloudlet::driver::{
        self,
        process::{Directory, Reference, StdReader},
    },
    WasmDriverState,
};

impl driver::process::Host for WasmDriverState {
    fn spawn_process(
        &mut self,
        command: String,
        args: Vec<String>,
        directory: Directory,
    ) -> Result<u32, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let process_dir = match &directory.reference {
            Reference::Controller => PathBuf::from(".").join(&directory.path),
            Reference::Data => {
                Storage::get_data_folder_for_driver(&driver.name).join(&directory.path)
            }
            Reference::Configs => {
                Storage::get_config_folder_for_driver(&driver.name).join(&directory.path)
            }
        };
        let command = Command::new(command)
            .args(args)
            .current_dir(process_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
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

    fn try_wait(&mut self, pid: u32) -> Result<Option<i32>, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut child_processes = driver
            .data
            .child_processes
            .write()
            .map_err(|_| "Failed to acquire write lock on child processes")?;

        if let Some(child) = child_processes.get_mut(&pid) {
            child
                .try_wait()
                .map_err(|error| format!("Failed to wait for child process: {}", error))
                .map(|status| {
                    if let Some(status) = status {
                        child_processes.remove(&pid);
                        Some(status.code().unwrap_or(0))
                    } else {
                        None
                    }
                })
        } else {
            Ok(None)
        }
    }

    fn read_line(&mut self, pid: u32, std: StdReader) -> Result<(u32, String), String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut child_processes = driver
            .data
            .child_processes
            .write()
            .map_err(|_| "Failed to acquire write lock on child processes")?;

        if let Some(child) = child_processes.get_mut(&pid) {
            match std {
                StdReader::Stdout => {
                    let stdout = child
                        .stdout
                        .as_mut()
                        .ok_or("Failed to open stdout of child process")?;
                    let mut buffer = String::new();
                    let mut reader = BufReader::new(stdout);
                    let bytes = reader.read_line(&mut buffer).map_err(|error| {
                        format!("Failed to read stdout of child process: {}", error)
                    })?;
                    Ok((bytes as u32, buffer))
                }
                StdReader::Stderr => {
                    let stderr = child
                        .stderr
                        .as_mut()
                        .ok_or("Failed to open stderr of child process")?;
                    let mut buffer = String::new();
                    let mut reader = BufReader::new(stderr);
                    let bytes = reader.read_line(&mut buffer).map_err(|error| {
                        format!("Failed to read stderr of child process: {}", error)
                    })?;
                    Ok((bytes as u32, buffer))
                }
            }
        } else {
            Err("Child process does not exist".to_string())
        }
    }

    fn read_to_end(&mut self, pid: u32, std: StdReader) -> Result<(u32, Vec<u8>), String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let mut child_processes = driver
            .data
            .child_processes
            .write()
            .map_err(|_| "Failed to acquire write lock on child processes")?;

        if let Some(child) = child_processes.get_mut(&pid) {
            match std {
                StdReader::Stdout => {
                    let output = child
                        .stdout
                        .as_mut()
                        .ok_or("Failed to open stdout of child process")?;
                    let mut buffer = Vec::new();
                    let bytes = output.read_to_end(&mut buffer).map_err(|error| {
                        format!("Failed to read stdout of child process: {}", error)
                    })?;
                    Ok((bytes as u32, buffer))
                }
                StdReader::Stderr => {
                    let output = child
                        .stderr
                        .as_mut()
                        .ok_or("Failed to open stderr of child process")?;
                    let mut buffer = Vec::new();
                    let bytes = output.read_to_end(&mut buffer).map_err(|error| {
                        format!("Failed to read stderr of child process: {}", error)
                    })?;
                    Ok((bytes as u32, buffer))
                }
            }
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
