mod driver;
mod network;
mod config;

use std::thread;
use std::time::Duration;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, ConfigBuilder, format_description, TerminalMode, TermLogger};
use crate::config::Config;
use crate::driver::{Driver, load_server_driver};
use crate::network::start_controller_server;

pub const VERSION: u16 = 1;

struct Controller {
    configuration: Config,
    driver: Driver
}

impl Controller {
    fn new(configuration: Config) -> Self {
        let driver = load_server_driver(&configuration);
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
        .set_time_format_custom(format_description!("[hour]:[minute]:[second].[subsecond]"))
        .build(), TerminalMode::Mixed, ColorChoice::Auto
    ).expect("Failed to init logging crate");
    info!("Starting cluster system version v{}...", VERSION);

    info!("Loading configuration...");
    let configuration = Config::new_filled();
    let controller = Controller::new(configuration);
    controller.start();
}