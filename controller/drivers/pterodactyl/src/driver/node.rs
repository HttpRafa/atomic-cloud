use colored::Colorize;
use std::{
    cell::UnsafeCell,
    rc::Rc,
    sync::{Mutex, MutexGuard},
    vec,
};

use crate::{
    error,
    exports::node::driver::bridge::{Address, Capabilities, GuestGenericNode, Server},
};

use super::{
    backend::{allocation::BAllocation, server::BServerFeatureLimits, Backend},
    PterodactylNodeWrapper,
};

impl GuestGenericNode for PterodactylNodeWrapper {
    fn new(name: String, id: Option<u32>, capabilities: Capabilities) -> Self {
        Self {
            inner: Rc::new(PterodactylNode {
                backend: UnsafeCell::new(None),
                id: id.unwrap(),
                name,
                capabilities,
                used_allocations: Mutex::new(vec![]),
            }),
        }
    }

    /* This method expects that the Pterodactyl Allocations are only accessed by one atomic cloud instance */
    fn allocate_addresses(&self, amount: u32) -> Result<Vec<Address>, String> {
        let mut used = self.inner.get_used_allocations();
        let allocations = self
            .get_backend()
            .get_free_allocations(&used, self.inner.id, amount)
            .iter()
            .map(|allocation| {
                used.push(allocation.clone());
                Address {
                    ip: allocation.ip.clone(),
                    port: allocation.port,
                }
            })
            .collect();
        Ok(allocations)
    }

    fn deallocate_addresses(&self, addresses: Vec<Address>) {
        self.inner.get_used_allocations().retain(|x| {
            !addresses
                .iter()
                .any(|address| *x.ip == address.ip && x.port == address.port)
        });
    }

    fn start_server(&self, server: Server) {
        // Check if a server with the same name is already exists
        if let Some(_server) = self.get_backend().get_server_by_name(&server.name) {
            // Just use the existing server and change its settings
        } else {
            let allocation = match self.inner.find_allocation(&server.allocation.addresses[0]) {
                Some(allocation) => allocation,
                None => {
                    error!(
                        "Allocation({:?}) not found for server {}",
                        &server.allocation.addresses[0], server.name
                    );
                    return;
                }
            };

            let mut egg = None;
            let mut startup = None;
            for value in server.allocation.deployment.settings.iter() {
                match value.key.as_str() {
                    "egg" => match value.value.parse::<u32>() {
                        Ok(id) => {
                            egg = Some(id);
                        }
                        Err(_) => {
                            error!("The egg setting must be a number!");
                        }
                    },
                    "startup" => {
                        startup = Some(value.value.clone());
                    }
                    _ => {}
                }
            }

            let mut missing = vec![];
            if egg.is_none() {
                missing.push("egg");
            }
            if startup.is_none() {
                missing.push("startup");
            }
            if !missing.is_empty() {
                error!(
                    "The following required settings to start the server are missing: {}",
                    missing.join(", ").red()
                );
                return;
            }

            // Create a new server
            self.get_backend().create_server(
                &server,
                self.inner.id,
                &allocation,
                egg.unwrap(),
                startup.unwrap().as_str(),
                BServerFeatureLimits {
                    databases: 0,
                    backups: 0,
                },
            );
        }
    }

    fn stop_server(&self, _server: Server) {}
}

pub struct PterodactylNode {
    /* Informations about the node */
    pub backend: UnsafeCell<Option<Rc<Backend>>>,
    pub id: u32,
    pub name: String,
    pub capabilities: Capabilities,

    pub used_allocations: Mutex<Vec<BAllocation>>,
}

impl PterodactylNode {
    fn get_used_allocations(&self) -> MutexGuard<Vec<BAllocation>> {
        // Safe as we are only run on the same thread
        self.used_allocations.lock().unwrap()
    }

    fn find_allocation(&self, address: &Address) -> Option<BAllocation> {
        self.get_used_allocations()
            .iter()
            .find(|allocation| allocation.ip == address.ip && allocation.port == address.port)
            .cloned()
    }
}
