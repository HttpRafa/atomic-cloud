use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
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
        plugin::system::data_types::Features,
    },
    info,
    node::{backend::Backend, InnerNode, Node},
};

pub mod config;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const FEATURES: Features = Features::NODE;

// Rc is used here to allow resorces to be shared between the plugin and the nodes
pub struct Pelican {
    /* Cloud Identification */
    identifier: String,

    /* Configuration */
    config: Rc<RefCell<Config>>,

    /* Nodes */
    nodes: RefCell<Vec<Rc<InnerNode>>>,
}

impl GuestPlugin for Pelican {
    fn new(identifier: String) -> Self {
        Self {
            identifier,
            config: Rc::new(RefCell::new(Config::default())), // Dummy config
            nodes: RefCell::new(vec![]),
        }
    }

    fn init(&self) -> Information {
        fn inner(own: &Pelican) -> Result<()> {
            // Load configuration
            {
                let config = Config::parse()?;
                own.config.replace(config);
            }
            Ok(())
        }

        Information {
            authors: AUTHORS.iter().map(|author| (*author).to_string()).collect(),
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
        unimplemented!()
    }

    fn init_node(
        &self,
        name: String,
        capabilities: Capabilities,
        controller: String,
    ) -> Result<GenericNode, ErrorMessage> {
        if let Some(value) = capabilities.child.as_ref() {
            let backend =
                Backend::new(&self.config.borrow(), value).map_err(|error| error.to_string())?;
            let node = Node::new(
                self.identifier.clone(),
                name.clone(),
                capabilities,
                controller,
                self.config.clone(),
                backend,
            );

            self.nodes.borrow_mut().push(node.0.clone());
            info!("Initialized node {}", name);
            Ok(GenericNode::new(node))
        } else {
            Err("Node lacks the required child capability".to_string())
        }
    }

    fn tick(&self) -> Result<(), ScopedErrors> {
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ScopedErrors> {
        Ok(())
    }
}
