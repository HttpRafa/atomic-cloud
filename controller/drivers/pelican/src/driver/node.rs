use std::sync::Arc;

use rand::Rng;

use crate::exports::node::driver::bridge::{Address, Capability, GuestGenericNode};

use super::PelicanNodeWrapper;

impl GuestGenericNode for PelicanNodeWrapper {
    fn new(name: String, capabilities: Vec<Capability>) -> Self {
        Self {
            inner: Arc::new(PelicanNode {
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
}

pub struct PelicanNode {
    pub name: String,
    pub capabilities: Vec<Capability>,
}