use std::{fs::{self, File}, path::PathBuf, process::exit};

use colored::Colorize;
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};

use crate::version::Version;

pub struct CloudInit;

impl CloudInit {
    pub fn init_logging(debug: bool, log_file: PathBuf) {
        if let Some(parent) = log_file.parent() {
            if !parent.exists() {
                if let Err(error) = fs::create_dir_all(parent) {
                    println!(
                        "Failed to create logs directory: {}",
                        &error
                    );
                    exit(1);
                }
            }
        }

        Self::init_logging_with_writeable(
            debug,
            File::create(log_file).expect("Failed to create log file"),
        );
    }

    pub fn init_logging_with_writeable(debug: bool, log_file: File) {
        if debug {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Error)
                        .build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    ConfigBuilder::new()
                        .set_location_level(LevelFilter::Error)
                        .build(),
                    log_file,
                ),
            ])
            .expect("Failed to init logging crate");
        } else {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Info,
                    ConfigBuilder::new().build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(LevelFilter::Info, ConfigBuilder::new().build(), log_file),
            ])
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
