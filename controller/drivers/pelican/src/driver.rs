use backend::Backend;
use colored::Colorize;

use crate::{error, exports::node::driver::bridge::{Capability, GuestGenericDriver, Information}, info};

mod backend;

const AUTHORS: [&str; 1] = ["HttpRafa"];
const VERSION: &str = "0.1.0";

pub struct Pelican {
    backend: Option<Backend>
}

impl GuestGenericDriver for Pelican {
    fn new() -> Self {
        Self {
            backend: Backend::new_filled()
        }
    }
    fn init(&self) -> Information {
        Information {
            authors: AUTHORS.map(|author|author.to_string()).to_vec(),
            version: VERSION.to_string(),
            ready: self.backend.is_some(),
        }
    }
    fn init_node(&self, name: String, capabilities: Vec<Capability>) -> bool {
        info!("Checking node {}", &name.blue());
        let _sub_node = if let Some(Capability::SubNode(value)) = capabilities.iter().find(|cap| matches!(cap, Capability::SubNode(_))) {
            value
        } else {
            error!("{} to check node {} because it lacks the required sub-node capability to use Pelican as the backend", "Failed".red(), &name.blue());
            return false;
        };
        true
    }
}