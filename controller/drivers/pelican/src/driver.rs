use std::cell::UnsafeCell;
use backend::Backend;
use colored::Colorize;

use crate::exports::node::driver::bridge::{Capability, GuestGenericDriver, Information};
use crate::info;

mod backend;

const AUTHORS: [&str; 1] = ["HttpRafa"];
const VERSION: &str = "0.1.0";

pub struct Pelican {
    backend: UnsafeCell<Option<Backend>>,
}

impl Pelican {
    fn get_backend(&self) -> &Backend {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.backend.get() }.as_ref().unwrap()
    }
}

impl GuestGenericDriver for Pelican {
    fn new() -> Self {
        Self {
            backend: UnsafeCell::new(None),
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

    fn init_node(&self, name: String, capabilities: Vec<Capability>) -> Option<String> {
        info!("Checking node {}", name.blue());

        if let Some(Capability::SubNode(ref value)) = capabilities.iter().find(|cap| matches!(cap, Capability::SubNode(_))) {
            if !self.get_backend().node_exists(value) {
                return Some("Node does not exist in the Pelican panel".to_string());
            }
            None
        } else {
            Some("Node lacks the required sub-node capability".to_string())
        }
    }
}