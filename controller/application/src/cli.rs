use std::path::Path;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use inquire::{required, Confirm, Select, Text};
use log::LevelFilter;

use crate::{
    config::{auto_complete::SimpleAutoComplete, validators::UnsignedValidator},
    driver::{DATA_DIRECTORY, DRIVERS_DIRECTORY},
    node::{Capability, Nodes},
};

#[derive(Parser)]
pub(super) struct Cli {
    #[arg(short, long, default_value_t = false)]
    debug: bool,
    #[arg(short, long, default_value_t = false)]
    generate_node: bool,
}

impl Cli {
    pub fn run(log_level: &mut LevelFilter) -> bool {
        let cli = Cli::parse();
        if cli.debug {
            *log_level = LevelFilter::Debug;
        }
        if cli.generate_node {
            if let Err(err) = Self::generate_node() {
                println!("{} {} failed to generate node: {}", ">".cyan(), "Failed".red(), &err);
            }
            return true;
        }
        false
    }

    fn generate_node() -> Result<()> {
        let mut capabilities = Vec::new();

        let name = Text::new("What is the name of the node?")
            .with_validator(required!())
            .with_autocomplete(SimpleAutoComplete::from_slices(vec!["home", "node1", "node2", "node3"]))
            .prompt()?;

        let drivers = Self::get_drivers()?;

        let driver = Text::new("What is the name of the driver you want to use? NOTE: This driver has to be installed into the drivers directory.")
            .with_validator(required!())
            .with_autocomplete(SimpleAutoComplete::from_strings(drivers))
            .prompt()?;

        match Select::new("What memory capabilities does the node have?", vec!["unlimited", "limited"]).prompt()? {
            "unlimited" => capabilities.push(Capability::UnlimitedMemory(true)),
            "limited" => {
                let memory = Text::new("How much memory does the node have in MiB?")
                    .with_validator(UnsignedValidator::new())
                    .with_validator(required!())
                    .with_autocomplete(SimpleAutoComplete::from_slices(vec!["1024", "2048", "4096", "8192", "16384", "32768"]))
                    .prompt()?;
                capabilities.push(Capability::LimitedMemory(memory.parse()?));
            },
            _ => unreachable!(),
        }

        if Confirm::new("Do you want to limit how many servers can be started on this node?").with_placeholder("y/n").prompt()? {
            let limit = Text::new("How many servers can be started on this node?")
                .with_validator(UnsignedValidator::new())
                .with_validator(required!())
                .with_autocomplete(SimpleAutoComplete::from_slices(vec!["5", "10", "15", "20", "25"]))
                .prompt()?;
            capabilities.push(Capability::MaxServers(limit.parse()?));
        }

        if Confirm::new("Some drivers can use sub-nodes if the underlying backend also uses the concept of nodes. Do you want to use sub-nodes?").with_placeholder("y/n").prompt()? {
            let sub_node = Text::new("What is the name of sub-node you want to use?")
                .with_validator(required!())
                .with_autocomplete(SimpleAutoComplete::from_slices(vec!["local", "home"]))
                .prompt()?;
            capabilities.push(Capability::SubNode(sub_node));
        }

        Nodes::create_node(&name, driver, capabilities)?;
        println!("{} Node {} has been created successfully!", ">".cyan(), &name.blue());
        Ok(())
    }

    fn get_drivers() -> Result<Vec<String>> {
        let drivers_directory = Path::new(DRIVERS_DIRECTORY);
        let mut drivers = Vec::new();
    
        if drivers_directory.exists() {
            for entry in drivers_directory.read_dir()? {
                let entry = entry?;
                if entry.file_name() == DATA_DIRECTORY {
                    continue;
                }
                if entry.file_type()?.is_dir() {
                    drivers.extend(
                        entry.path().read_dir()?
                            .filter_map(Result::ok)
                            .filter_map(|driver| driver.path().file_stem().map(|stem| stem.to_string_lossy().to_string()))
                    );
                }
            }
        }
        Ok(drivers)
    }    
}