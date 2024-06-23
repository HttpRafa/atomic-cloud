use std::{cell::UnsafeCell, sync::Arc};

use crate::exports::node::driver::bridge::{Address, Capabilities, GuestGenericNode, Server};

use super::{backend::Backend, PterodactylNodeWrapper};

impl GuestGenericNode for PterodactylNodeWrapper {
    fn new(name: String, id: Option<u32>, capabilities: Capabilities) -> Self {
        Self {
            inner: Arc::new(PterodactylNode {
                backend: UnsafeCell::new(None),
                id: id.unwrap(),
                name,
                capabilities,
            }),
        }
    }

    /* This method expects that the Pterodactyl Allocations are only accessed by one atomic cloud instance */
    fn allocate_addresses(&self, amount: u32) -> Result<Vec<Address>, String> {
        Ok(self.get_backend().get_free_allocations(self.inner.id, amount).iter().map(|allocation| Address {
            ip: allocation.ip.clone(),
            port: allocation.port,
        }).collect())
    }

    fn deallocate_addresses(&self, _addresses: Vec<Address>) {}

    fn start_server(&self, _server: Server) {
    }

    fn stop_server(&self, _server: Server) {
    }
}

pub struct PterodactylNode {
    pub backend: UnsafeCell<Option<Arc<Backend>>>,
    pub id: u32,
    pub name: String,
    pub capabilities: Capabilities,
}