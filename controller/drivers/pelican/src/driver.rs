use crate::{error, exports::node::driver::bridge::{Capability, GuestGenericDriver, Information}, info};
use colored::Colorize;

mod backend;

const AUTHORS: [&str; 1] = ["HttpRafa"];
const VERSION: &str = "0.1.0";

pub struct Pelican {
    
}

impl GuestGenericDriver for Pelican {
    fn new() -> Self {
        Self {}
    }
    fn init(&self) -> Information {
        Information {
            authors: AUTHORS.map(|author|author.to_string()).to_vec(),
            version: VERSION.to_string(),
            ready: true,
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