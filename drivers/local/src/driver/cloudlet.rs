use std::{cell::UnsafeCell, rc::Rc, sync::RwLock};

use common::{allocator::NumberAllocator, name::TimedName};
use unit::LocalUnit;

use crate::{
    error,
    exports::cloudlet::driver::bridge::{
        Address, Capabilities, GuestGenericCloudlet, RemoteController, Retention, Unit,
        UnitProposal,
    },
    storage::Storage,
};

use super::{config::Config, template::Templates, LocalCloudletWrapper};

pub mod unit;

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
                templates: UnsafeCell::new(None),
                port_allocator: UnsafeCell::new(None),
                units: RwLock::new(vec![]),
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

    fn start_unit(&self, unit: Unit) {
        let spec = &unit.allocation.spec;
        let name =
            TimedName::new_no_identifier(&unit.name, spec.disk_retention == Retention::Permanent);

        let template = spec
            .settings
            .iter()
            .find(|s| s.key == "template")
            .map(|s| s.value.clone());
        if template.is_none() {
            error!(
                "The following required settings to start the unit are missing: <red>template</>"
            );
            return;
        }

        let template = match self
            .get_templates()
            .read()
            .expect("Failed to lock templates")
            .get_template_by_name(template.as_ref().unwrap())
        {
            Some(value) => value,
            None => {
                error!(
                    "Failed to start unit <blue>{}</>: Template <blue>{}</> not found",
                    name.get_name(),
                    &template.unwrap()
                );
                return;
            }
        };

        let folder = Storage::get_unit_folder(&name, &spec.disk_retention);
        if !folder.exists() {
            if let Err(error) = template.copy_to_folder(&folder) {
                error!(
                    "Failed to start unit <blue>{}</>: Failed to copy template: <red>{}</>",
                    name.get_name(),
                    error
                );
                return;
            }
        }

        match LocalUnit::start(&name, &folder, &template) {
            Ok(unit) => self
                .inner
                .units
                .write()
                .expect("Failed to lock units")
                .push(unit),
            Err(error) => error!(
                "Failed to start unit <blue>{}</>: <red>{}</>",
                name.get_raw_name(),
                error
            ),
        }
    }

    fn restart_unit(&self, _unit: Unit) {}

    fn stop_unit(&self, _unit: Unit) {}
}

pub struct LocalCloudlet {
    /* Informations about the cloudlet */
    pub _name: String,
    pub config: UnsafeCell<Option<Rc<Config>>>,
    pub controller: RemoteController,

    /* Templates */
    pub templates: UnsafeCell<Option<Rc<RwLock<Templates>>>>,

    /* Dynamic Resources */
    pub port_allocator: UnsafeCell<Option<Rc<RwLock<NumberAllocator<u16>>>>>,
    pub units: RwLock<Vec<LocalUnit>>,
}
