use std::time::Instant;

use colored::Colorize;
use common::init::CloudInit;
use common::version::{Stage, Version};
use log::info;

use crate::config::Config;
use crate::controller::Controller;

mod config;
mod controller;
mod network;

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const VERSION: Version = Version {
    major: 0,
    minor: 1,
    patch: 1,
    stage: Stage::Alpha,
};

fn main() {
    CloudInit::init_logging();
    CloudInit::print_ascii_art("Atomic Cloud", &VERSION, &AUTHORS);

    let start_time = Instant::now();
    info!(
        "Starting cluster system version {}...",
        format!("v{}", VERSION).blue()
    );
    info!("{} configuration...", "Loading".green());

    let configuration = Config::new_filled();
    let controller = Controller::new(configuration);
    info!("Loaded cluster system in {:.2?}", start_time.elapsed());
    controller.start();
}
