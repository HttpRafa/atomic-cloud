#![warn(clippy::all, clippy::pedantic)]
#![feature(let_chains)]

use application::Cli;
use color_eyre::eyre::Result;

mod application;
mod storage;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = Cli::new().await?.run(terminal).await;
    ratatui::restore();
    result
}
