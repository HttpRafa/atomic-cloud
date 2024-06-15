use std::cell::UnsafeCell;
use std::sync::{Arc, Mutex};
use backend::Backend;
use colored::Colorize;
use node::PterodactylNode;

use crate::exports::node::driver::bridge::{Capabilities, GenericNode, GuestGenericDriver, GuestGenericNode, Information};
use crate::info;

pub mod node;

mod backend;

const AUTHORS: [&str; 1] = ["HttpRafa"];
const VERSION: &str = "0.1.0-alpha";

pub struct Pterodactyl {
    backend: UnsafeCell<Option<Backend>>,

    /* Nodes that this driver handles */
    nodes: Mutex<Vec<Arc<PterodactylNode>>>,
}

impl Pterodactyl {
    fn get_backend(&self) -> &Backend {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.backend.get() }.as_ref().unwrap()
    }
}

impl GuestGenericDriver for Pterodactyl {
    fn new() -> Self {
        Self {
            backend: UnsafeCell::new(None),
            nodes: Mutex::new(Vec::new()),
        }
    }

    fn init(&self) -> Information {
        unsafe { *self.backend.get() = Backend::new_filled(); }
        Information {
            authors: AUTHORS.iter().map(|&author| author.to_string()).collect(),
            version: VERSION.to_string(),
            ready: unsafe { &*self.backend.get() }.is_some(),
        }
    }

    fn init_node(&self, name: String, capabilities: Capabilities) -> Result<GenericNode, String> {
        info!("Checking node {}", name.blue());

        if let Some(value) = capabilities.sub_node.as_ref() {
            if !self.get_backend().node_exists(&value) {
                return Err("Node does not exist in the Pterodactyl panel".to_string());
            }
            let wrapper = PterodactylNodeWrapper::new(name, capabilities);
            // Add node to nodes
            let mut nodes = self.nodes.lock().expect("Failed to get lock on nodes");
            nodes.push(wrapper.inner.clone());
            Ok(GenericNode::new(wrapper))
        } else {
            Err("Node lacks the required sub-node capability".to_string())
        }
    }
}

pub struct PterodactylNodeWrapper {
    pub inner: Arc<PterodactylNode>,
}