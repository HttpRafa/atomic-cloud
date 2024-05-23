mod driver;
mod network;
mod config;
mod version;

use std::thread;
use std::time::Duration;
use colored::Colorize;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, ConfigBuilder, TerminalMode, TermLogger};
use crate::config::Config;
use crate::driver::load_server_driver;
use crate::driver::lua::LuaDriver;
use crate::network::start_controller_server;
use crate::version::Version;

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const VERSION: Version = Version {
    major: 1,
    minor: 0,
    patch: 0
};

struct Controller {
    configuration: Config,
    driver: LuaDriver
}

impl Controller {
    async fn new(configuration: Config) -> Self {
        let driver = load_server_driver(&configuration).await;
        Controller {
            configuration,
            driver,
        }
    }
    fn start(&self) {
        info!("Starting networking stack...");
        start_controller_server(&self.configuration);

        loop {
            thread::sleep(Duration::new(5, 0));
        }
    }
}

#[tokio::main]
async fn main() {
    TermLogger::init(
        LevelFilter::Debug, ConfigBuilder::new()
            .set_location_level(LevelFilter::Error)
            .build(), TerminalMode::Mixed, ColorChoice::Auto
    ).expect("Failed to init logging crate");
    print_ascii_art();
    info!("Starting cluster system version v{}...", VERSION);

    info!("Loading configuration...");
    let configuration = Config::new_filled();
    let controller = Controller::new(configuration).await;
    controller.start();
}

fn print_ascii_art() {
    println!("{}{}", r"   _   _                  _".blue(), r"          ___ _                 _".cyan());
    println!("{}{}", r"  /_\ | |_ ___  _ __ ___ (_) ___".blue(), r"    / __\ | ___  _   _  __| |".cyan());
    println!("{}{}", r" //_\\| __/ _ \| '_ ` _ \| |/ __|".blue(), r"  / /  | |/ _ \| | | |/ _` |".cyan());
    println!("{}{}", r"/  _  \ || (_) | | | | | | | (__".blue(), r"  / /___| | (_) | |_| | (_| |".cyan());
    println!("{}{}", r"\_/ \_/\__\___/|_| |_| |_|_|\___|".blue(), r" \____/|_|\___/ \__,_|\__,_|".cyan());
    println!();
    println!("«{}» {} | {} by {}", "*".blue(), "Atomic Cloud".blue(), format!("v{}", VERSION.major).blue(), AUTHORS.join(", ").blue());
    println!();
}