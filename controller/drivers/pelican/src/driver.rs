use std::cell::UnsafeCell;

use backend::Backend;
use colored::Colorize;

use crate::{
    error,
    exports::node::driver::bridge::{Capability, GuestGenericDriver, Information},
    info, warn,
};

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

    fn init_node(&self, name: String, capabilities: Vec<Capability>) -> bool {
        info!("Checking node {}", name.blue());

        if let Some(Capability::SubNode(ref value)) = capabilities.iter().find(|cap| matches!(cap, Capability::SubNode(_))) {
            if !self.get_backend().node_exists(value) {
                warn!("{} to check node {} because it does not exist in the Pelican panel", "Failed".red(), name.blue());
                return false;
            }
            true
        } else {
            error!(
                "{} to check node {} because it lacks the required sub-node capability to use Pelican as the backend",
                "Failed".red(),
                name.blue()
            );
            false
        }
    }
}
