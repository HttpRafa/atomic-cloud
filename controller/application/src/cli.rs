use std::fs::read;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use inquire::{required, Confirm, Select, Text};
use log::error;

use crate::{config::validators::UnsignedValidator, node::{Capability, Nodes}};

#[derive(Parser)]
pub(super) struct Cli {
    #[arg(long, default_value_t = false)]
    generate_node: bool,
}

impl Cli {
    pub fn run() -> bool {
        let cli = Cli::parse();
        if cli.generate_node {
            if let Err(error) = Self::generate_node() {
                error!("{} failed to generate node: {}", "Failed".red(), &error);
            }
            return true;
        }
        false
    }
    fn generate_node() -> Result<()> {
        let mut capabilities = Vec::new();

        let name = Text::new("What is the name of the node?")
            .with_validator(required!())
            .prompt()?;

        let driver = Text::new("What is the name of the driver you want to use? NOTE: This driver has to be installed into the drivers directory.")
            .with_validator(required!())
            .prompt()?;

        match Select::new("What memory capabilities does the node have?", vec!["unlimited", "limited"]).prompt()? {
            "unlimited" => {
                capabilities.push(Capability::UnlimitedMemory(true));
            }
            "limited" => {
                let memory = Text::new("How much memory does the node have in MiB?")
                    .with_validator(UnsignedValidator::new())
                    .with_validator(required!())
                    .prompt()?;
                capabilities.push(Capability::LimitedMemory(memory.parse()?));
            }
            _ => unreachable!(),
        }

        if Confirm::new("Do you want to limit how many servers can be started on this node?").prompt()? {
            let limit = Text::new("How many servers can be started on this node?")
                .with_validator(UnsignedValidator::new())
                .with_validator(required!())
                .prompt()?;
            capabilities.push(Capability::MaxServers(limit.parse()?));
        }

        if Confirm::new("Some drivers can use sub-nodes if the underlying backend also uses the concept of nodes. Do you want to use sub-nodes?").prompt()? {
            capabilities.push(Capability::SubNode(Text::new("What is the name of sub-node you want to use?").with_validator(required!()).prompt()?));
        }

        Nodes::create_node(name, driver, capabilities)?;
        Ok(())
    }
}