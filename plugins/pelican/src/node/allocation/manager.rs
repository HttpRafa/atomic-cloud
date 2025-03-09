use std::{cell::RefCell, collections::HashMap};

use anyhow::Result;
use common::name::TimedName;

use crate::{
    generated::exports::plugin::system::bridge::{Address, DiskRetention, ServerProposal},
    node::{backend::allocation::data::BAllocation, InnerNode},
    warn,
};

pub struct AllocationManager {
    allocations: HashMap<u16, BAllocation>,
}

impl AllocationManager {
    pub fn init() -> RefCell<Self> {
        RefCell::new(Self {
            allocations: HashMap::new(),
        })
    }

    pub fn allocate(&mut self, node: &InnerNode, server: ServerProposal) -> Result<Vec<Address>> {
        let required = server.resources.ports as usize;

        if matches!(server.spec.disk_retention, DiskRetention::Permanent) {
            let name = TimedName::new(&node.identifier, &server.name, true);

            // Check if a server with the same name is already exists
            if let Some(backend_server) = node.backend.get_server_by_name(&name) {
                // Get the allocations that are already used by this server
                let mut allocations = node
                    .backend
                    .get_allocations_by_server(&backend_server.identifier);

                if (allocations.1.len() + 1) != required {
                    warn!("The server {} has a different amount of addresses than the panel has allocated. This may cause issues.", server.name);
                    // TODO: Add a way to fix this
                }

                allocations.1.insert(0, allocations.0); // Add primary allocation to the list
                allocations.1.iter().for_each(|address| {
                    self.allocations.insert(address.port, address.into());
                });
                return Ok(allocations.1.into_iter().map(|x| x.into()).collect());
            }
        }

        let allocations = node
            .backend
            .get_free_allocations(&self.allocations, required)
            .iter()
            .map(|allocation| {
                self.allocations.insert(allocation.port, allocation.clone());
                Address {
                    host: allocation.get_host().clone(),
                    port: allocation.port,
                }
            })
            .collect();
        Ok(allocations)
    }

    pub fn free(&mut self, addresses: Vec<Address>) {
        for address in addresses {
            if self.allocations.remove(&address.port).is_none() {
                warn!(
                    "Failed to free address, because it was never marked as used: {:?}",
                    address
                );
            }
        }
    }
}
