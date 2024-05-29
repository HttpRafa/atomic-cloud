use std::time::Instant;

use colored::Colorize;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode, TermLogger};

use crate::config::Config;
use crate::controller::Controller;
use crate::version::Version;

mod driver;
mod network;
mod config;
mod node;
mod controller;

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const VERSION: Version = Version {
    major: 1,
    minor: 0,
    patch: 0,
};

#[tokio::main]
async fn main() {
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new()
            .set_location_level(LevelFilter::Error)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto
    ).expect("Failed to init logging crate");

    print_ascii_art();

    let start_time = Instant::now();
    info!("Starting cluster system version {}...", format!("v{}", VERSION).blue());
    info!("Loading configurations...");

    let configuration = Config::new_filled();
    let mut controller = Controller::new(configuration).await;
    info!("Loaded cluster system in {:.2?}", start_time.elapsed());
    controller.start().await;
}

fn print_ascii_art() {
    println!("{}{}", r"   _   _                  _".blue(), r"          ___ _                 _".cyan());
    println!("{}{}", r"  /_\ | |_ ___  _ __ ___ (_) ___".blue(), r"    / __\ | ___  _   _  __| |".cyan());
    println!("{}{}", r" //_\\| __/ _ \| '_ ` _ \| |/ __|".blue(), r"  / /  | |/ _ \| | | |/ _` |".cyan());
    println!("{}{}", r"/  _  \ || (_) | | | | | | | (__".blue(), r"  / /___| | (_) | |_| | (_| |".cyan());
    println!("{}{}", r"\_/ \_/\__\___/|_| |_| |_|_|\___|".blue(), r" \____/|_|\___/ \__,_|\__,_|".cyan());
    println!();
    println!("«{}» {} | {} by {}", "*".blue(), "Atomic Cloud".blue(), format!("v{}", VERSION).blue(), AUTHORS.join(", ").blue());
    println!();
}

mod version {
    use std::fmt::{Display, Formatter};

    pub struct Version {
        pub major: u16,
        pub minor: u16,
        pub patch: u16,
    }

    impl Display for Version {
        fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}