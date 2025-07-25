use std::{
    collections::HashMap,
    process::{self, Stdio},
};

use anyhow::{Result, anyhow};
use simplelog::debug;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    spawn,
    sync::mpsc::{Receiver, channel},
    task::JoinHandle,
};
use wasmtime::component::Resource;

use crate::application::plugin::runtime::wasm::{
    PluginState,
    config::Permissions,
    generated::plugin::system::{
        self,
        process::ExitStatus,
        types::{Directory, ErrorMessage},
    },
};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

const STREAM_BUFFER: usize = 128;

pub struct ProcessBuilder {
    command: String,
    args: Vec<String>,
    environment: HashMap<String, String>,
    directory: Option<Directory>,
}
pub struct Process {
    child: Child,
    streams: Streams,
}

struct Streams {
    tasks: (JoinHandle<()>, JoinHandle<()>),
    stdin: Option<BufWriter<ChildStdin>>,
    receiver: Receiver<String>,
}

impl Streams {
    pub fn new(
        stdin: Option<ChildStdin>,
        stdout: Option<ChildStdout>,
        stderr: Option<ChildStderr>,
    ) -> Self {
        let stdin = stdin.map(BufWriter::new);
        let stdout = stdout.map(BufReader::new);
        let stderr = stderr.map(BufReader::new);

        let (sender, receiver) = channel(STREAM_BUFFER);

        let stdout_task = spawn(Self::handle_stream(stdout, sender.clone(), "stdout"));
        let stderr_task = spawn(Self::handle_stream(stderr, sender, "stderr"));

        Self {
            tasks: (stdout_task, stderr_task),
            stdin,
            receiver,
        }
    }

    pub fn abort(&self) {
        self.tasks.0.abort();
        self.tasks.1.abort();
    }

    async fn handle_stream<R>(
        mut reader: Option<BufReader<R>>,
        sender: tokio::sync::mpsc::Sender<String>,
        stream_name: &str,
    ) where
        R: tokio::io::AsyncRead + Unpin,
    {
        if let Some(reader) = reader.as_mut() {
            let mut buffer = String::new();
            loop {
                match reader.read_line(&mut buffer).await {
                    Ok(0) => break, // EOF reached
                    Ok(_) => {
                        if let Err(error) = sender.send(buffer.clone()).await {
                            debug!("Failed to send {} line: {}", stream_name, error);
                            break;
                        }
                        buffer.clear();
                    }
                    Err(error) => {
                        debug!("Error reading from {}: {}", stream_name, error);
                        break;
                    }
                }
            }
        }
    }
}

impl system::process::Host for PluginState {}

impl system::process::HostProcessBuilder for PluginState {
    async fn new(&mut self, command: String) -> Result<Resource<ProcessBuilder>> {
        Ok(self.resources.push(ProcessBuilder {
            command,
            args: Vec::new(),
            environment: HashMap::new(),
            directory: None,
        })?)
    }
    async fn args(&mut self, instance: Resource<ProcessBuilder>, args: Vec<String>) -> Result<()> {
        self.resources.get_mut(&instance)?.args.extend(args);
        Ok(())
    }
    async fn environment(
        &mut self,
        instance: Resource<ProcessBuilder>,
        environment: Vec<(String, String)>,
    ) -> Result<()> {
        self.resources
            .get_mut(&instance)?
            .environment
            .extend(environment);
        Ok(())
    }
    async fn directory(
        &mut self,
        instance: Resource<ProcessBuilder>,
        directory: Directory,
    ) -> Result<()> {
        self.resources.get_mut(&instance)?.directory = Some(directory);
        Ok(())
    }
    async fn spawn(
        &mut self,
        instance: Resource<ProcessBuilder>,
    ) -> Result<Result<Resource<Process>, ErrorMessage>> {
        // Check if the plugin has permissions
        if !self.permissions.contains(Permissions::ALLOW_PROCESS) {
            return Err(anyhow!(
                "Plugin tried to spawn a process without the required permissions"
            ));
        }

        let builder = self.resources.get(&instance)?;
        debug!("Spawning process: {} {:?}", builder.command, builder.args);

        let mut command = Command::new(&builder.command);
        if let Some(directory) = &builder.directory {
            command.current_dir(Self::get_directory(&self.name, directory));
        }
        command
            .args(&builder.args)
            .envs(&builder.environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => return Ok(Err(format!("Failed to spawn process: {error}"))),
        };

        let streams = Streams::new(child.stdin.take(), child.stdout.take(), child.stderr.take());
        Ok(Ok(self.resources.push(Process { child, streams })?))
    }
    async fn drop(&mut self, instance: Resource<ProcessBuilder>) -> Result<()> {
        self.resources.delete(instance)?;
        Ok(())
    }
}

impl system::process::HostProcess for PluginState {
    async fn kill(&mut self, instance: Resource<Process>) -> Result<Result<(), ErrorMessage>> {
        Ok(self
            .resources
            .get_mut(&instance)?
            .child
            .kill()
            .await
            .map_err(|error| format!("Failed to kill process: {error}")))
    }
    async fn try_wait(
        &mut self,
        instance: Resource<Process>,
    ) -> Result<Result<Option<ExitStatus>, ErrorMessage>> {
        Ok(self
            .resources
            .get_mut(&instance)?
            .child
            .try_wait()
            .map(|status| status.map(create_exit_status))
            .map_err(|error| format!("Failed to try waiting for process: {error}")))
    }
    async fn read_lines(&mut self, instance: Resource<Process>) -> Result<Vec<String>> {
        let process = self.resources.get_mut(&instance)?;
        let mut lines = vec![];
        while let Ok(line) = process.streams.receiver.try_recv() {
            lines.push(line);
        }
        Ok(lines)
    }
    async fn write_all(
        &mut self,
        instance: Resource<Process>,
        data: Vec<u8>,
    ) -> Result<Result<(), ErrorMessage>> {
        if let Some(stdin) = &mut self.resources.get_mut(&instance)?.streams.stdin {
            return Ok(stdin
                .write_all(&data)
                .await
                .map_err(|error| format!("Failed to write to process: {error}")));
        }
        Ok(Err("Process stdin is not available".to_string()))
    }
    async fn flush(&mut self, instance: Resource<Process>) -> Result<Result<(), ErrorMessage>> {
        if let Some(stdin) = &mut self.resources.get_mut(&instance)?.streams.stdin {
            return Ok(stdin
                .flush()
                .await
                .map_err(|error| format!("Failed to write to process: {error}")));
        }
        Ok(Err("Process stdin is not available".to_string()))
    }
    async fn drop(&mut self, instance: Resource<Process>) -> Result<()> {
        let process = self.resources.delete(instance)?;
        process.streams.abort();
        Ok(())
    }
}

#[cfg(unix)]
fn create_exit_status(status: process::ExitStatus) -> ExitStatus {
    if let Some(code) = status.code() {
        ExitStatus::Code(code)
    } else if let Some(signal) = status.signal() {
        ExitStatus::Signal(signal)
    } else {
        ExitStatus::Unknown
    }
}

#[cfg(windows)]
fn create_exit_status(status: process::ExitStatus) -> ExitStatus {
    if let Some(code) = status.code() {
        ExitStatus::Code(code)
    } else {
        ExitStatus::Unknown
    }
}
