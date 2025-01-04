use application::Cli;
use args::Args;
use clap::Parser;
use common::init::CloudInit;
use storage::Storage;

mod application;
mod args;
mod storage;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];

#[tokio::main]
async fn main() {
    let args = Args::parse();
    CloudInit::init_logging(args.debug, Storage::get_latest_log_file());
    CloudInit::print_ascii_art("Atomic Cloud CLI", &VERSION, &AUTHORS);

    let mut cli = Cli::new().await;
    cli.start().await
}
