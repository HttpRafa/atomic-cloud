use std::{cell::UnsafeCell, rc::Rc, sync::RwLock};

use common::allocator::NumberAllocator;

use crate::exports::cloudlet::driver::bridge::{
    Address, Capabilities, GuestGenericCloudlet, RemoteController, Unit, UnitProposal,
};

use super::{config::Config, LocalCloudletWrapper};

impl GuestGenericCloudlet for LocalCloudletWrapper {
    fn new(
        _cloud_identifier: String,
        _name: String,
        _id: Option<u32>,
        _capabilities: Capabilities,
        controller: RemoteController,
    ) -> Self {
        Self {
            inner: Rc::new(LocalCloudlet {
                _name,
                config: UnsafeCell::new(None),
                controller,
                port_allocator: UnsafeCell::new(None),
            }),
        }
    }

    fn allocate_addresses(&self, unit: UnitProposal) -> Result<Vec<Address>, String> {
        let amount = unit.resources.addresses;

        let mut ports = Vec::with_capacity(amount as usize);
        let mut allocator = self
            .get_port_allocator()
            .write()
            .expect("Failed to lock port allocator");
        for _ in 0..amount {
            if let Some(port) = allocator.allocate() {
                ports.push(Address {
                    host: self.inner.controller.address.clone(),
                    port,
                });
            } else {
                return Err("Failed to allocate ports".to_string());
            }
        }

        Ok(ports)
    }

    fn deallocate_addresses(&self, addresses: Vec<Address>) {
        let mut allocator = self
            .get_port_allocator()
            .write()
            .expect("Failed to lock port allocator");
        for address in addresses {
            allocator.release(address.port);
        }
    }

    fn start_unit(&self, _unit: Unit) {}

    fn restart_unit(&self, _unit: Unit) {}

    fn stop_unit(&self, _unit: Unit) {}
}

pub struct LocalCloudlet {
    /* Informations about the cloudlet */
    pub _name: String,
    pub config: UnsafeCell<Option<Rc<Config>>>,
    pub controller: RemoteController,

    /* Dynamic Resources */
    pub port_allocator: UnsafeCell<Option<Rc<RwLock<NumberAllocator<u16>>>>>,
}
