use std::sync::Arc;

use rand::Rng;

use crate::exports::node::driver::bridge::{Address, Capability, GuestGenericNode, Server};

use super::PterodactylNodeWrapper;

impl GuestGenericNode for PterodactylNodeWrapper {
    fn new(name: String, capabilities: Vec<Capability>) -> Self {
        Self {
            inner: Arc::new(PterodactylNode {
                name,
                capabilities,
            }),
        }
    }

    fn allocate_addresses(&self, amount: u32) -> Result<Vec<Address>, String> {
        let mut addresses = Vec::new();
        let mut random = rand::thread_rng();
        for _ in 0..amount {
            addresses.push(Address {
                ip: format!("{}.{}.{}.{}", random.gen_range(1..255), random.gen_range(0..255), random.gen_range(0..255), random.gen_range(0..255)),
                port: random.gen_range(25565..65535),
            });
        }
        Ok(addresses)
    }

    fn deallocate_addresses(&self, _addresses: Vec<Address>) {}

    fn start_server(&self, _server: Server) {
    }

    fn stop_server(&self, _server: Server) {
    }
}

pub struct PterodactylNode {
    pub name: String,
    pub capabilities: Vec<Capability>,
}