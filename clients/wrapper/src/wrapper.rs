use std::process::{exit, Command, Stdio};

use colored::Colorize;
use log::{error, info};

pub struct Wrapper {
    pub program: String,
    pub args: Vec<String>,
}

impl Wrapper {
    pub fn new() -> Wrapper {
        let mut args = std::env::args();
        if args.len() < 2 {
            error!("Usage: {} <program> [args...]", args.next().unwrap());
            exit(1);
        }
        let mut args = args.skip(1);
        let program = args.next().unwrap();
        Self {
            program,
            args: args.collect(),
        }
    }
    pub fn start(&mut self) {
        info!("{} child process...", "Starting".green());
        let mut child = match Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::inherit())
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                error!("{} to start child process: {}", "Failed".red(), error);
                exit(1);
            }
        };
        let status = match child.wait() {
            Ok(status) => status,
            Err(error) => {
                error!("{} to wait for child process: {}", "Failed".red(), error);
                exit(1);
            }
        };

        info!("Child process {} with status {}", "exited".red(), status);
    }
}
