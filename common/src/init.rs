use std::{
    fs::{self, File},
    path::PathBuf,
    process::exit,
};

use colored::Colorize;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};

use crate::version::Version;

pub struct CloudInit;

impl CloudInit {
    pub fn init_logging(debug: bool, minimal: bool, log_file: PathBuf) {
        if let Some(parent) = log_file.parent() {
            if !parent.exists() {
                if let Err(error) = fs::create_dir_all(parent) {
                    println!("Failed to create logs directory: {}", &error);
                    exit(1);
                }
            }
        }

        Self::init_logging_with_writeable(
            debug,
            minimal,
            File::create(log_file).expect("Failed to create log file"),
        );
    }

    pub fn init_logging_with_writeable(debug: bool, minimal: bool, log_file: File) {
        let mut config = ConfigBuilder::new();
        if minimal {
            config.set_max_level(LevelFilter::Off);
            config.set_time_level(LevelFilter::Off);
        }
        if debug {
            config.set_location_level(LevelFilter::Error);
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    config.build(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(LevelFilter::Debug, config.build(), log_file),
            ])
            .expect("Failed to init logging crate");
        } else {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Info,
                    config.build(),
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
            format!("v{version}").blue(),
            authors.join(", ").blue()
        );
        println!();
    }
}
