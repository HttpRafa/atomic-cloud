use std::{
    cell::UnsafeCell,
    rc::Rc,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Result;
use common::{allocator::NumberAllocator, name::TimedName, tick::TickResult};
use server::LocalUnit;

use crate::{
    node::plugin::types::{ErrorMessage, ScopedError, ScopedErrors},
    error,
    exports::node::plugin::bridge::{
        Address, Capabilities, GuestGenericCloudlet, RemoteController, Retention, Unit,
        UnitProposal,
    },
    info,
    storage::Storage,
};

use super::{config::Config, template::Templates, LocalCloudletWrapper};

pub mod server;

impl LocalCloudlet {
    pub fn tick(&self) -> Result<(), ScopedErrors> {
        let mut servers = self.get_servers_mut();
        let mut errors = ScopedErrors::new();
        servers.retain_mut(|server| match server.tick() {
            Ok(result) => result == TickResult::Ok,
            Err(err) => {
                errors.push(ScopedError {
                    scope: server.name.get_raw_name().to_string(),
                    message: err.to_string(),
                });
                true
            }
        });
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

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
                servers: RwLock::new(vec![]),
            }),
        }
    }

    fn tick(&self) -> Result<(), ScopedErrors> {
        self.inner.tick()
    }

    fn allocate_addresses(&self, server: UnitProposal) -> Result<Vec<Address>, ErrorMessage> {
        let amount = server.resources.addresses;

        let mut ports = Vec::with_capacity(amount as usize);
        let mut allocator = self
            .inner
            .get_port_allocator()
            .write()
            .expect("Failed to lock port allocator");
        for _ in 0..amount {
            if let Some(port) = allocator.allocate() {
                ports.push(Address {
                    host: self.inner.get_config().address.clone(),
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
            .inner
            .get_port_allocator()
            .write()
            .expect("Failed to lock port allocator");
        for address in addresses {
            allocator.release(address.port);
        }
    }

    fn start_server(&self, server: Unit) {
        let spec = &server.allocation.spec;
        let name =
            TimedName::new_no_identifier(&server.name, spec.disk_retention == Retention::Permanent);

        let template = match self
            .inner
            .get_templates()
            .read()
            .expect("Failed to lock templates")
            .get_template_by_name(&spec.image)
        {
            Some(template) => template,
            None => {
                error!(
                    "Template <blue>{}</> not found for server <blue>{}</>",
                    &spec.image,
                    name.get_name()
                );
                return;
            }
        };

        let folder = Storage::get_server_folder(&name, &spec.disk_retention);
        if !folder.exists() {
            if let Err(err) = template.copy_to_folder(&folder) {
                error!(
                    "Failed to copy template for server <blue>{}</>: <red>{}</>",
                    name.get_name(),
                    err
                );
                return;
            }
        }

        let mut local_server = LocalUnit::new(self, server, &name, template);
        if let Err(err) = local_server.start() {
            error!(
                "Failed to start server <blue>{}</>: <red>{}</>",
                name.get_raw_name(),
                err
            );
            return;
        }

        info!(
            "Successfully <green>created</> child process for server <blue>{}</>",
            name.get_raw_name()
        );
        self.inner.get_servers_mut().push(local_server);
    }

    fn restart_server(&self, server: Unit) {
        let mut servers = self.inner.get_servers_mut();
        if let Some(local_server) = servers
            .iter_mut()
            .find(|u| u.name.get_raw_name() == server.name)
        {
            if let Err(err) = local_server.restart() {
                error!(
                    "<red>Failed</> to restart server <blue>{}</>: <red>{}</>",
                    server.name, err
                );
                return;
            }
            info!(
                "Child process of server <blue>{}</> is <yellow>restarting</>",
                server.name
            );
        } else {
            error!("<red>Failed</> to restart server <blue>{}</>: Unit was <red>never started</> by this plugin", server.name);
        }
    }

    fn stop_server(&self, server: Unit) {
        let mut servers = self.inner.get_servers_mut();
        if let Some(local_server) = servers
            .iter_mut()
            .find(|u| u.name.get_raw_name() == server.name)
        {
            if server.allocation.spec.disk_retention == Retention::Temporary {
                if let Err(err) = local_server.kill() {
                    error!(
                        "<red>Failed</> to stop server <blue>{}</>: <red>{}</>",
                        server.name, err
                    );
                    return;
                }
                info!(
                    "Child process of server <blue>{}</> was <red>killed</>",
                    server.name
                );
            } else {
                if let Err(err) = local_server.stop() {
                    error!(
                        "<red>Failed</> to stop server <blue>{}</>: <red>{}</>",
                        server.name, err
                    );
                    return;
                }
                info!(
                    "Child process of server <blue>{}</> is <red>stopping</>",
                    server.name
                );
            }
        } else {
            error!("<red>Failed</> to stop server <blue>{}</>: Unit was <red>never started</> by this plugin", server.name);
        }
    }
}

pub struct LocalCloudlet {
    /* Informations about the node */
    _name: String,
    pub config: UnsafeCell<Option<Rc<Config>>>,
    controller: RemoteController,

    /* Templates */
    pub templates: UnsafeCell<Option<Rc<RwLock<Templates>>>>,

    /* Dynamic Resources */
    pub port_allocator: UnsafeCell<Option<Rc<RwLock<NumberAllocator<u16>>>>>,
    servers: RwLock<Vec<LocalUnit>>,
}

impl LocalCloudlet {
    /* Dispose */
    pub fn try_exit(&self, force: bool) -> Result<TickResult, ScopedErrors> {
        if force {
            let mut servers = self.get_servers_mut();
            let mut errors = ScopedErrors::new();
            for server in servers.iter_mut() {
                if let Err(error) = server.kill() {
                    errors.push(ScopedError {
                        scope: server.name.get_raw_name().to_string(),
                        message: error.to_string(),
                    });
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }
        }
        match self.tick() {
            Ok(()) => {
                if self.get_servers().is_empty() {
                    Ok(TickResult::Drop)
                } else {
                    Ok(TickResult::Ok)
                }
            }
            Err(errors) => Err(errors),
        }
    }

    fn get_config(&self) -> &Rc<Config> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.config.get() }.as_ref().unwrap()
    }
    fn get_templates(&self) -> &Rc<RwLock<Templates>> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.templates.get() }.as_ref().unwrap()
    }
    fn get_port_allocator(&self) -> &Rc<RwLock<NumberAllocator<u16>>> {
        // Safe as we are only borrowing the reference immutably
        unsafe { &*self.port_allocator.get() }.as_ref().unwrap()
    }
    fn get_servers(&self) -> RwLockReadGuard<Vec<LocalUnit>> {
        // Safe as we are only run on the same thread
        self.servers.read().unwrap()
    }
    fn get_servers_mut(&self) -> RwLockWriteGuard<Vec<LocalUnit>> {
        // Safe as we are only run on the same thread
        self.servers.write().unwrap()
    }
}
