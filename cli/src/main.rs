use common::{
    init::CloudInit,
    version::{Stage, Version},
};

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const VERSION: Version = Version {
    major: 0,
    minor: 1,
    patch: 0,
    stage: Stage::Alpha,
};

#[tokio::main]
async fn main() {
    CloudInit::init_logging();
    CloudInit::print_ascii_art("Atomic Cloud CLI", &VERSION, &AUTHORS);
}
