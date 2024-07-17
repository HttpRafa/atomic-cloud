use std::{
    process::{exit, Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use colored::Colorize;
use log::{error, info};

const TICK_RATE: u64 = 1;

pub struct Wrapper {
    /* Immutable */
    pub program: String,
    pub args: Vec<String>,

    /* Runtime State */
    running: Arc<AtomicBool>,

    /* Child */
    process: Option<Child>,
}

impl Wrapper {
    pub fn new() -> Wrapper {
        let mut args = std::env::args();
        if args.len() < 2 {
            error!(
                "Usage: {} <{}> [{}]",
                args.next().unwrap().blue(),
                "program".blue(),
                "args...".blue()
            );
            exit(1);
        }
        let mut args = args.skip(1);
        let program = args.next().unwrap();
        Self {
            program,
            args: args.collect(),
            running: Arc::new(AtomicBool::new(true)),
            process: None,
        }
    }

    pub fn start(&mut self) {
        // Set up signal handlers
        let running = self.running.clone();
        ctrlc::set_handler(move || {
            info!("{} signal received. Stopping...", "Interrupt".red());
            running.store(false, Ordering::Relaxed);
        })
        .expect("Failed to set Ctrl+C handler");

        // Start child process
        self.start_child();

        let tick_duration = Duration::from_millis(1000 / TICK_RATE);
        while self.running.load(Ordering::Relaxed) {
            let start_time = Instant::now();
            self.tick();

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                thread::sleep(tick_duration - elapsed_time);
            }
        }
    }

    fn tick(&mut self) {
        // Request stop when child process stopped
        if let Some(status) = self
            .process
            .as_mut()
            .and_then(|child| child.try_wait().ok().flatten())
        {
            info!(
                "Child process {} with status {}",
                "exited".red(),
                format!("{}", status).blue()
            );
            info!("Wrapper stop requested. Stopping...");
            self.running.store(false, Ordering::Relaxed);
        }
    }

    fn start_child(&mut self) {
        info!("{} child process...", "Starting".green());
        self.process = Some(
            match Command::new(&self.program)
                .args(&self.args)
                .stdin(Stdio::inherit())
                .spawn()
            {
                Ok(child) => child,
                Err(error) => {
                    error!("{} to start child process: {}", "Failed".red(), error);
                    exit(1);
                }
            },
        )
    }
}
