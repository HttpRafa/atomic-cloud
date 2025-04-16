#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::ref_option)]
#![feature(binary_heap_drain_sorted)]

use anyhow::Result;
use application::Controller;
use clap::{ArgAction, Parser};
use common::{error::FancyError, init::CloudInit};
use config::Config;
use simplelog::info;
use storage::Storage;
use tokio::time::Instant;

mod application;
mod config;
mod network;
mod resource;
mod storage;
mod task;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];

#[tokio::main]
async fn main() {
    async fn run() -> Result<()> {
        let arguments = Arguments::parse();
        CloudInit::init_logging(arguments.debug, false, Storage::latest_log_file());
        CloudInit::print_ascii_art("Atomic Cloud", &VERSION, &AUTHORS);

        let beginning = Instant::now();
        info!("Starting cloud version v{}...", VERSION);
        info!("Initializing controller...");

        let mut controller = Controller::init(Config::parse().await?).await?;
        info!("Loaded cloud in {:.2?}", beginning.elapsed());
        controller.run().await?;

        Ok(())
    }

    if let Err(error) = run().await {
        FancyError::print_fancy(&error, true);
    }
}

#[derive(Parser)]
struct Arguments {
    #[clap(short, long, help = "Enable debug mode", action = ArgAction::SetTrue)]
    debug: bool,
}
