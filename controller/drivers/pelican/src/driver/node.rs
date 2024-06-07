use std::sync::Arc;

use rand::Rng;

use crate::exports::node::driver::bridge::{Capability, GuestGenericNode};

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

    fn allocate_ports(&self, amount: u32) -> Result<Vec<u32>, String> {
        let mut ports = Vec::new();
        let mut random = rand::thread_rng();
        for _ in 0..amount {
            ports.push(random.gen_range(25565..65535));
        }
        Ok(ports)
    }
}

pub struct PelicanNode {
    pub name: String,
    pub capabilities: Vec<Capability>,
}