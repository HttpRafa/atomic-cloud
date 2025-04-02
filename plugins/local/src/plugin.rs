use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, Result};
use common::allocator::NumberAllocator;
use config::Config;

use crate::{
    error,
    generated::{
        exports::plugin::system::{
            bridge::{
                Capabilities, ErrorMessage, GuestPlugin, Information, Listener as GenericListener,
                Node as GenericNode, ScopedErrors,
            },
            event::Events,
        },
        plugin::system::{data_types::Features, file::remove_dir_all},
    },
    info,
    listener::Listener,
    node::{InnerNode, Node},
    storage::Storage,
    template::manager::TemplateManager,
};

pub mod config;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const FEATURES: Features = Features::all();

// Rc is used here to allow resorces to be shared between the plugin and the nodes
pub struct Local {
    /* Configuration */
    config: Rc<RefCell<Config>>,

    /* Shared */
    allocator: Rc<RefCell<NumberAllocator<u16>>>,
    templates: Rc<RefCell<TemplateManager>>,

    /* Nodes */
    nodes: RefCell<Vec<Rc<InnerNode>>>,
}

impl GuestPlugin for Local {
    fn new(_: String) -> Self {
        Self {
            config: Rc::new(RefCell::new(Config::default())), // Dummy config
            allocator: Rc::new(RefCell::new(NumberAllocator::new(0..10))), // Dummy allocator
            templates: Rc::new(RefCell::new(TemplateManager::default())),
            nodes: RefCell::new(vec![]),
        }
    }

    fn init(&self) -> Information {
        fn inner(own: &Local) -> Result<()> {
            // Delete temporary files if they exist
            if Storage::temporary_directory(false).exists() {
                info!("Removing temporary files");
                remove_dir_all(&Storage::create_temporary_directory())
                    .map_err(|error| anyhow!(error))?;
            }

            // Load configuration
            {
                let config = Config::parse()?;
                own.allocator
                    .replace(NumberAllocator::new(config.range().clone()));
                own.config.replace(config);
            }

            // Load templates
            {
                let mut templates = own.templates.borrow_mut();
                templates.init()?;
                templates.run_prepare()?
            }
            Ok(())
        }

        Information {
            authors: AUTHORS.iter().map(|author| author.to_string()).collect(),
            version: VERSION.to_string(),
            features: FEATURES,
            ready: if let Err(error) = inner(self) {
                error!("Failed to initialize plugin: {}", error);
                false
            } else {
                true
            },
        }
    }

    fn init_listener(&self) -> (Events, GenericListener) {
        (Events::empty(), GenericListener::new(Listener()))
    }

    fn init_node(
        &self,
        name: String,
        capabilities: Capabilities,
        controller: String,
    ) -> Result<GenericNode, ErrorMessage> {
        let node = Node::new(
            name.clone(),
            capabilities,
            controller,
            self.config.clone(),
            self.allocator.clone(),
            self.templates.clone(),
        );

        self.nodes.borrow_mut().push(node.0.clone());
        info!("Initialized node {}", name);
        Ok(GenericNode::new(node))
    }

    fn tick(&self) -> Result<(), ScopedErrors> {
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ScopedErrors> {
        Ok(())
    }
}
