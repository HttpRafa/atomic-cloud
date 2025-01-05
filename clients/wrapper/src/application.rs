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
use transfer::Transfers;
use user::Users;

use crate::args::Args;

mod detection;
mod heart;
mod network;
mod process;
mod transfer;
mod user;

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
    users: Users,
    transfers: Transfers,
    connection: Arc<CloudConnection>,
}

impl Wrapper {
    pub async fn new(args: Args) -> Wrapper {
        let mut args = args.command.into_iter();
        if args.len() == 0 {
            error!("No program specified. Use --help for more information.");
            exit(1);
        }
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
            users: Users::new(connection.clone()),
            transfers: Transfers::from_env(connection.clone()),
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

        // Tell the controller we are a new client
        self.connection.send_reset().await.expect("Failed to send reset signal to controller");

        // Start child process
        self.start_child();

        // Subscribe to network events
        self.transfers.subscribe().await;

        let mut tick_interval = interval(Duration::from_millis(1000 / TICK_RATE));
        while self.running.load(Ordering::Relaxed) {
            select! {
                _ = tick_interval.tick() => self.tick().await,
                _ = self.process.as_mut().unwrap().stdout_tick(&mut self.users) => {},
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

        if let Some(process) = &mut self.process {
            // Process transfers
            self.transfers.tick(process, &self.users).await;

            // Request stop when child process stopped
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
