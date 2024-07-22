use std::{process::{exit, Stdio}, sync::Arc};

use colored::Colorize;
use log::{error, info};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStdout, Command},
};

use super::{detection::RegexDetector, network::CloudConnection};

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
    connection: Arc<CloudConnection>,

    /* StdOut Reader */
    stdout: BufReader<ChildStdout>,
}

impl ManagedProcess {
    pub fn start(program: &str, args: &[String], detector: RegexDetector, connection: Arc<CloudConnection>) -> Self {
        info!("{} child process...", "Starting".green());
        info!("-> {} {}", program.blue(), args.join(" "));

        let mut process = match Command::new(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                error!("{} to start child process: {}", "Failed".red(), error);
                exit(1);
            }
        };

        let stdout = BufReader::new(
            process
                .stdout
                .take()
                .expect("Failed to open stdout of child"),
        );

        Self { process, state: State::Starting, detector, connection, stdout }
    }

    pub async fn tick(&mut self) -> bool {
        if let Some(status) = self.process.try_wait().ok().flatten() {
            info!(
                "Child process {} with {}",
                "exited".red(),
                format!("{}", status).blue()
            );
            self.handle_state_change(State::Stopped).await;
            return true;
        }
        false
    }

    pub async fn stdout_tick(&mut self) {
        let mut buffer = String::new();
        if self.stdout.read_line(&mut buffer).await.unwrap() > 0 {
            let line = buffer.trim();
            println!("{} {}", "#".blue(), line);
            if self.state == State::Starting && self.detector.is_started(line) {
                self.handle_state_change(State::Running).await;
            } else if self.state == State::Running && self.detector.is_stopping(line) {
                self.handle_state_change(State::Stopping).await;
            }
        }
    }

    pub async fn kill_if_running(&mut self) {
        if self.process.try_wait().ok().flatten().is_none() {
            info!("{} child process...", "Stopping".red());
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

    async fn handle_state_change(&mut self, state: State) {
        self.state = state;
        match self.state {
            State::Running => {
                info!("The child process has {} successfully", "started".green());
                if let Err(error) = self.connection.mark_ready().await {
                    error!("Failed to report ready state to controller: {}", error);
                }
            }
            State::Stopping => {
                info!("The child process is {}", "stopping".red());
            }
            _ => {}
        }
    }
}
