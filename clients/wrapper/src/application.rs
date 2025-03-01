use std::{
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use detection::RegexDetector;
use heart::Heart;
use network::CloudConnection;
use process::ManagedProcess;
use simplelog::{error, info};
use tokio::select;
use transfer::Transfers;
use user::Users;

use crate::args::Args;

mod detection;
mod heart;
mod network;
mod process;
mod transfer;
mod user;

// TODO: Change this to a configuration value
static BEAT_INTERVAL: Duration = Duration::from_secs(10);

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

    pub async fn start(&mut self) -> Result<()> {
        // Set up signal handlers
        let running = Arc::downgrade(&self.running);
        ctrlc::set_handler(move || {
            info!("Interrupt signal received. Stopping...");
            if let Some(running) = running.upgrade() {
                running.store(false, Ordering::Relaxed);
            }
        })
        .expect("Failed to set Ctrl+C handler");

        // Start child process
        self.start_child();

        // Subscribe to network events
        self.transfers.subscribe().await;

        while self.running.load(Ordering::Relaxed) {
            self.tick().await;
        }

        // Kill child process
        if let Some(mut process) = self.process.take() {
            process.kill_if_running().await;
        }

        info!("All done! Bye :)");
        Ok(())
    }

    pub fn request_stop(&self) {
        info!("Wrapper stop requested. Stopping...");
        self.running.store(false, Ordering::Relaxed);
    }

    async fn tick(&mut self) {
        if let Some(process) = &mut self.process {
            select! {
                message = self.transfers.next_message() => self.transfers.handle_message(message, &mut process.stdin, &self.users).await,
                _ = self.heart.wait_for_beat() => self.heart.beat().await,
                result = process.process.wait() => if process.handle_process_exit(result).await { self.request_stop() },
                line = process.stdout.next_line() => process.handle_stdout_line(line, &mut self.users).await,
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
