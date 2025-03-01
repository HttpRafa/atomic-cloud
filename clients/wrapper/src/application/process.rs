use std::process::{exit, Stdio};

use simplelog::{error, info};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStdin, ChildStdout, Command},
};

use crate::application::detection::Detection;

use super::{detection::RegexDetector, network::CloudConnectionHandle, user::Users};

#[derive(PartialEq)]
pub enum State {
    Starting,
    Running,
    Stopping,
    Stopped,
}

pub struct ManagedProcess {
    /* Process */
    process: Child,
    state: State,

    /* Detection and Network */
    detector: RegexDetector,
    connection: CloudConnectionHandle,

    /* StdOut Reader */
    stdout: BufReader<ChildStdout>,

    /* StdIn Writer */
    stdin: BufWriter<ChildStdin>,
}

impl ManagedProcess {
    pub fn start(
        program: &str,
        args: &[String],
        detector: RegexDetector,
        connection: CloudConnectionHandle,
    ) -> Self {
        info!("Starting child process...");
        info!("-> {} {}", program, args.join(" "));

        let mut process = match Command::new(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                error!("Failed to start child process: {}", error);
                exit(1);
            }
        };

        let stdout = BufReader::new(
            process
                .stdout
                .take()
                .expect("Failed to open stdout of child"),
        );
        let stdin = BufWriter::new(process.stdin.take().expect("Failed to open stdin of child"));

        Self {
            process,
            state: State::Starting,
            detector,
            connection,
            stdout,
            stdin,
        }
    }

    pub async fn tick(&mut self) -> bool {
        if let Some(status) = self.process.try_wait().ok().flatten() {
            info!("Child process exited with {}", status);
            self.handle_state_change(State::Stopped).await;
            return true;
        }
        false
    }

    pub async fn stdout_tick(&mut self, users: &mut Users) {
        let mut buffer = String::new();
        if self.stdout.read_line(&mut buffer).await.unwrap() > 0 {
            let line = buffer.trim();
            println!("# {}", line);
            match self.detector.detect(line) {
                Detection::Started => {
                    self.handle_state_change(State::Running).await;
                }
                Detection::Stopping => {
                    self.handle_state_change(State::Stopping).await;
                }
                Detection::UserConnected(user) => {
                    if user.name.is_none() || user.uuid.is_none() {
                        return;
                    }
                    users
                        .handle_connect(user.name.unwrap(), user.uuid.unwrap())
                        .await;
                }
                Detection::UserDisconnected(user) => {
                    if user.name.is_none() {
                        return;
                    }
                    users.handle_disconnect(user.name.unwrap()).await;
                }
                Detection::None => { /* Do nothing */ }
            }
        }
    }

    pub async fn kill_if_running(&mut self) {
        if self.process.try_wait().ok().flatten().is_none() {
            info!("Stopping child process...");
            self.process
                .kill()
                .await
                .expect("Failed to kill child process");
            self.process
                .wait()
                .await
                .expect("Failed to wait for child process");
        }
        self.handle_state_change(State::Stopped).await;
    }

    pub async fn write_to_stdin(&mut self, message: &str) {
        self.stdin.write_all(message.as_bytes()).await.unwrap();
        self.stdin.flush().await.unwrap();
    }

    async fn handle_state_change(&mut self, state: State) {
        if self.state == state {
            return;
        }

        match state {
            State::Running => {
                info!("The child process has started successfully");
                if let Err(error) = self.connection.set_running().await {
                    error!("Failed to report state to controller: {}", error);
                }
                if let Err(error) = self.connection.set_ready(true).await {
                    error!("Failed to report state to controller: {}", error);
                }
            }
            State::Stopping => {
                info!("The child process is stopping");
                if let Err(error) = self.connection.set_ready(false).await {
                    error!("Failed to report state to controller: {}", error);
                }
            }
            State::Stopped => {
                if let Err(error) = self.connection.request_stop().await {
                    error!("Failed to request hard from controller: {}", error);
                }
            }
            _ => { /* Do nothing */ }
        }
        self.state = state;
    }
}
