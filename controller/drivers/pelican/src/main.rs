#![no_main]

use colored::Colorize;
use exports::node::driver::bridge::{Guest, Information, Node};
use log::{info, warn, LevelFilter};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use wit_bindgen::generate;

generate!({
    world: "driver",
    path: "../../structure/wit/"
});

const AUTHORS: [&str; 1] = ["HttpRafa"];
const VERSION: &str = "0.1.0";

struct Pelican {}

impl Guest for Pelican {
    fn init() -> Information {
        TermLogger::init(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_location_level(LevelFilter::Error)
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Always
        ).expect("Failed to init logging crate");
        Information {
            authors: AUTHORS.map(|author|author.to_string()).to_vec(),
            version: VERSION.to_string(),
        }
    }
    fn init_node(_node: Node) -> bool {
        true
    }
}

export!(Pelican);