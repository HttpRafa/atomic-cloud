use std::{cell::UnsafeCell, rc::Rc, sync::{Mutex, MutexGuard}};

use crate::exports::node::driver::bridge::{Address, Capabilities, GuestGenericNode, Server};

use super::{backend::Backend, PterodactylNodeWrapper};

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
        let allocations = self.get_backend().get_free_allocations(&used, self.inner.id, amount).iter().map(|allocation| {
            used.push(Address {
                ip: allocation.ip.clone(),
                port: allocation.port,
            });
            Address {
                ip: allocation.ip.clone(),
                port: allocation.port,
            }
        }).collect();
        Ok(allocations)
    }

    fn deallocate_addresses(&self, addresses: Vec<Address>) {
        self.inner.get_used_allocations().retain(|x| !addresses.iter().any(|address| *x.ip == address.ip && x.port == address.port));
    }

    fn start_server(&self, _server: Server) {
    }

    fn stop_server(&self, _server: Server) {
    }
}

pub struct PterodactylNode {
    /* Informations about the node */
    pub backend: UnsafeCell<Option<Rc<Backend>>>,
    pub id: u32,
    pub name: String,
    pub capabilities: Capabilities,

    pub used_allocations: Mutex<Vec<Address>>,
}

impl PterodactylNode {
    fn get_used_allocations(&self) -> MutexGuard<Vec<Address>> {
        // Safe as we are only run on the same thread
        self.used_allocations.lock().unwrap()
    }
}