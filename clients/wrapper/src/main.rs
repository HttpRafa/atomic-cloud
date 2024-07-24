use colored::Colorize;
use common::{
    init::CloudInit,
    version::{Stage, Version},
};
use log::info;
use wrapper::Wrapper;

mod wrapper;

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const VERSION: Version = Version {
    major: 0,
    minor: 1,
    patch: 3,
    stage: Stage::Alpha,
};

#[tokio::main]
async fn main() {
    CloudInit::init_logging();
    CloudInit::print_ascii_art("Atomic Cloud Wrapper", &VERSION, &AUTHORS);

    info!("{} wrapper...", "Starting".green());
    let mut wrapper = Wrapper::new().await;
    wrapper.start().await;
}
