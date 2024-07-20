use backend::Backend;
use colored::Colorize;
use node::PterodactylNode;
use std::cell::UnsafeCell;
use std::rc::Rc;
use std::sync::Mutex;

use crate::exports::node::driver::bridge::{
    Capabilities, GenericNode, GuestGenericDriver, GuestGenericNode, Information,
};
use crate::{error, info};

pub mod node;

mod backend;

const AUTHORS: [&str; 1] = ["HttpRafa"];
const VERSION: &str = "0.1.0-alpha";

pub struct Pterodactyl {
    /* Cloud Identification */
    cloud_identifier: String,

    /* Backend */
    backend: UnsafeCell<Option<Rc<Backend>>>,

    /* Nodes that this driver handles */
    nodes: Mutex<Vec<Rc<PterodactylNode>>>,
}

impl Pterodactyl {
    fn get_backend(&self) -> &Rc<Backend> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.backend.get() }.as_ref().unwrap()
    }
}

impl GuestGenericDriver for Pterodactyl {
    fn new(cloud_identifier: String) -> Self {
        Self {
            cloud_identifier,
            backend: UnsafeCell::new(None),
            nodes: Mutex::new(Vec::new()),
        }
    }

    fn init(&self) -> Information {
        let backend = Backend::new_filled_and_resolved();
        if let Err(error) = &backend {
            error!(
                "Failed to initialize Pterodactyl driver: {}",
                error.to_string().red()
            );
        }
        unsafe { *self.backend.get() = backend.ok().map(Rc::new) };
        Information {
            authors: AUTHORS.iter().map(|&author| author.to_string()).collect(),
            version: VERSION.to_string(),
            ready: unsafe { &*self.backend.get() }.is_some(),
        }
    }

    fn init_node(
        &self,
        name: String,
        capabilities: Capabilities,
        controller_address: String,
    ) -> Result<GenericNode, String> {
        if let Some(value) = capabilities.sub_node.as_ref() {
            if let Some(node) = self.get_backend().get_node_by_name(value) {
                let wrapper = PterodactylNodeWrapper::new(
                    self.cloud_identifier.clone(),
                    name.clone(),
                    Some(node.id),
                    capabilities,
                    controller_address.clone(),
                );
                unsafe { *wrapper.inner.backend.get() = Some(self.get_backend().clone()) }
                // Add node to nodes
                let mut nodes = self.nodes.lock().expect("Failed to get lock on nodes");
                nodes.push(wrapper.inner.clone());
                info!(
                    "Node {}[{}] was {}",
                    name.blue(),
                    format!("{}", node.id).blue(),
                    "added".green()
                );
                Ok(GenericNode::new(wrapper))
            } else {
                Err("Node does not exist in the Pterodactyl panel".to_string())
            }
        } else {
            Err("Node lacks the required sub-node capability".to_string())
        }
    }
}

pub struct PterodactylNodeWrapper {
    pub inner: Rc<PterodactylNode>,
}

impl PterodactylNodeWrapper {
    fn get_backend(&self) -> &Rc<Backend> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.inner.backend.get() }.as_ref().unwrap()
    }
}
