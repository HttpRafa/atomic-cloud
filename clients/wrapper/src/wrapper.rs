use std::{
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use colored::Colorize;
use detection::RegexDetector;
use heart::Heart;
use log::{error, info};
use network::CloudConnection;
use process::ManagedProcess;
use tokio::{select, time::interval};

mod detection;
mod heart;
mod network;
mod process;

const TICK_RATE: u64 = 1;

// TODO: Change this to a configuration value
static BEAT_INTERVAL: Duration = Duration::from_secs(5);

pub struct Wrapper {
    /* Immutable */
    pub program: String,
    pub args: Vec<String>,

    /* Runtime State */
    running: Arc<AtomicBool>,
    process: Option<ManagedProcess>,

    /* Accessed frequently */
    heart: Heart,
    connection: Arc<CloudConnection>,
}

impl Wrapper {
    pub async fn new() -> Wrapper {
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

        let connection = CloudConnection::from_env();
        if let Err(error) = connection.connect().await {
            error!("Failed to connect to cloud: {}", error);
            exit(1);
        }

        Self {
            program,
            args: args.collect(),
            running: Arc::new(AtomicBool::new(true)),
            process: None,
            heart: Heart::new(BEAT_INTERVAL, connection.clone()),
            connection,
        }
    }

    pub async fn start(&mut self) {
        // Set up signal handlers
        let running = Arc::downgrade(&self.running);
        ctrlc::set_handler(move || {
            info!("{} signal received. Stopping...", "Interrupt".red());
            if let Some(running) = running.upgrade() {
                running.store(false, Ordering::Relaxed);
            }
        })
        .expect("Failed to set Ctrl+C handler");

        // Start child process
        self.start_child();

        let mut tick_interval = interval(Duration::from_millis(1000 / TICK_RATE));
        while self.running.load(Ordering::Relaxed) {
            select! {
                _ = tick_interval.tick() => self.tick().await,
                _ = self.process.as_mut().unwrap().stdout_tick() => {},
            }
        }

        // Kill child process
        if let Some(mut process) = self.process.take() {
            process.kill_if_running().await;
        }

        info!("All {}! Bye :)", "done".green());
    }

    pub fn request_stop(&self) {
        info!("Wrapper stop requested. {}...", "Stopping".red());
        self.running.store(false, Ordering::Relaxed);
    }

    async fn tick(&mut self) {
        // Heartbeat
        self.heart.tick().await;

        // Request stop when child process stopped
        if let Some(process) = &mut self.process {
            if process.tick().await {
                self.request_stop();
            }
        }
    }

    fn start_child(&mut self) {
        self.process = Some(ManagedProcess::start(
            &self.program,
            &self.args,
            RegexDetector::from_env(),
            self.connection.clone(),
        ));
    }
}
