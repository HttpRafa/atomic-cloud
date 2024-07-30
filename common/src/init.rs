use colored::Colorize;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

use crate::version::Version;

pub struct CloudInit;

impl CloudInit {
    pub fn init_logging(debug: bool) {
        if debug {
            TermLogger::init(
                LevelFilter::Debug,
                ConfigBuilder::new()
                    .set_location_level(LevelFilter::Error)
                    .build(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )
            .expect("Failed to init logging crate");
        } else {
            TermLogger::init(
                LevelFilter::Info,
                ConfigBuilder::new().build(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )
            .expect("Failed to init logging crate");
        }
    }

    pub fn print_ascii_art(application: &str, version: &Version, authors: &[&str]) {
        println!(
            "{}{}",
            r"   _   _                  _".blue(),
            r"          ___ _                 _".cyan()
        );
        println!(
            "{}{}",
            r"  /_\ | |_ ___  _ __ ___ (_) ___".blue(),
            r"    / __\ | ___  _   _  __| |".cyan()
        );
        println!(
            "{}{}",
            r" //_\\| __/ _ \| '_ ` _ \| |/ __|".blue(),
            r"  / /  | |/ _ \| | | |/ _` |".cyan()
        );
        println!(
            "{}{}",
            r"/  _  \ || (_) | | | | | | | (__".blue(),
            r"  / /___| | (_) | |_| | (_| |".cyan()
        );
        println!(
            "{}{}",
            r"\_/ \_/\__\___/|_| |_| |_|_|\___|".blue(),
            r" \____/|_|\___/ \__,_|\__,_|".cyan()
        );
        println!();
        println!(
            "«{}» {} | {} by {}",
            "*".blue(),
            application.blue(),
            format!("v{}", version).blue(),
            authors.join(", ").blue()
        );
        println!();
    }
}
