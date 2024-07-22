use std::process::{exit, Stdio};

use colored::Colorize;
use log::{error, info};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStdout, Command},
};

pub struct ManagedProcess {
    /* Process */
    process: Child,

    /* StdOut Reader */
    stdout: BufReader<ChildStdout>,
}

impl ManagedProcess {
    pub fn start(program: &str, args: &[String]) -> Self {
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

        Self { process, stdout }
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

    pub async fn stdout_tick(&mut self) {
        let mut buffer = String::new();
        if self.stdout.read_line(&mut buffer).await.unwrap() > 0 {
            let line = buffer.trim();
            println!("# {}", line);
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
    }
}
