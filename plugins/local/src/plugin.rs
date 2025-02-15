use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Result};
use common::allocator::NumberAllocator;
use config::Config;

use crate::{
    error, generated::{exports::plugin::system::bridge::{
        Capabilities, ErrorMessage, Features, GenericNode, GuestGenericPlugin, Information,
        ScopedErrors,
    }, plugin::system::file::remove_dir_all}, info, storage::Storage
};

pub mod config;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const FEATURES: Features = Features::all();

pub struct Local {
    identifier: String,

    // Using .unwrap() is safe here because the value is always set by the time it is accessed (after the plugin is initialized)
    config: RefCell<Config>,
    allocator: Rc<RefCell<NumberAllocator<u16>>>,
}

impl GuestGenericPlugin for Local {
    async fn new(identifier: String) -> Self {
        Self {
            identifier,
            config: RefCell::new(Config::default()), // Dummy config
            allocator: Rc::new(RefCell::new(NumberAllocator::new(0..10))), // Dummy allocator
        }
    }

    async fn init(&self) -> Information {
        async fn inner(own: &Local) -> Result<()> {
            // Delete temporary files if they exist
            if Storage::temporary_directory(false).exists() {
                info!("Removing temporary files");
                remove_dir_all(&Storage::create_temporary_directory()).await.map_err(|error| anyhow!(error))?;
            }

            // Load configuration
            {
                let config = Config::parse()?;
                own.allocator.replace(NumberAllocator::new(config.ports().clone()));
                own.config.replace(config);
            }
            Ok(())
        }

        Information {
            authors: AUTHORS.iter().map(|author| author.to_string()).collect(),
            version: VERSION.to_string(),
            features: FEATURES,
            ready: if let Err(error) = inner(self).await {
                error!("Failed to initialize plugin: {}", error);
                false
            } else {
                true
            },
        }
    }

    async fn init_node(
        &self,
        name: String,
        capabilities: Capabilities,
        controller: String,
    ) -> Result<GenericNode, ErrorMessage> {
        todo!()
    }

    async fn tick(&self) -> Result<(), ScopedErrors> {
        todo!()
    }

    async fn shutdown(&self) -> Result<(), ScopedErrors> {
        todo!()
    }
}
