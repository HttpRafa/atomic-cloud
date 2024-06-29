use colored::Colorize;
use server::{PanelServer, ServerName};
use std::{
    cell::UnsafeCell,
    rc::Rc,
    sync::{Mutex, MutexGuard},
    vec,
};

use crate::{
    error,
    exports::node::driver::bridge::{Address, Capabilities, GuestGenericNode, Retention, Server},
    info,
};

use super::{
    backend::{
        allocation::BAllocation,
        server::{BServerEgg, BServerFeatureLimits},
        Backend,
    },
    PterodactylNodeWrapper,
};

pub mod server;

impl GuestGenericNode for PterodactylNodeWrapper {
    fn new(
        cloud_identifier: String,
        name: String,
        id: Option<u32>,
        capabilities: Capabilities,
    ) -> Self {
        Self {
            inner: Rc::new(PterodactylNode {
                cloud_identifier,
                backend: UnsafeCell::new(None),
                id: id.unwrap(),
                name,
                capabilities,
                allocations: Mutex::new(vec![]),
                servers: Mutex::new(vec![]),
            }),
        }
    }

    /* This method expects that the Pterodactyl Allocations are only accessed by one atomic cloud instance */
    fn allocate_addresses(&self, amount: u32) -> Result<Vec<Address>, String> {
        let mut used = self.inner.get_allocations();
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
        self.inner.get_allocations().retain(|x| {
            !addresses
                .iter()
                .any(|address| *x.ip == address.ip && x.port == address.port)
        });
    }

    fn start_server(&self, server: Server) {
        let deployment = &server.allocation.deployment;
        let name = ServerName::new(
            &self.inner.cloud_identifier,
            &server.name,
            &deployment.disk_retention,
        );

        // Check if a server with the same name is already exists
        if let Some(_server) = self.get_backend().get_server_by_name(&name) {
            // Just use the existing server and change its settings
            info!(
                "Server {} already exists on the panel, updating settings",
                server.name
            );
        } else {
            let allocations = server
                .allocation
                .addresses
                .iter()
                .map_while(|address| match self.inner.find_allocation(address) {
                    Some(allocation) => Some(allocation),
                    None => {
                        error!(
                            "Allocation({:?}) not found for server {}",
                            &server.allocation.addresses[0], server.name
                        );
                        None
                    }
                })
                .collect::<Vec<_>>();

            let mut egg = None;
            let mut startup = None;
            for value in deployment.settings.iter() {
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
            if let Some(server) = self.get_backend().create_server(
                &name,
                self.inner.id,
                &server.allocation,
                &allocations,
                BServerEgg {
                    id: egg.unwrap(),
                    startup: startup.unwrap(),
                },
                BServerFeatureLimits {
                    databases: 0,
                    backups: 0,
                },
            ) {
                info!("Server {} {} on the panel", server.name, "created".green());
                self.inner
                    .get_servers()
                    .push(PanelServer::new(server.id, name));
            }
        }
    }

    fn stop_server(&self, server: Server) {
        if let Some(backend_server) = self.inner.find_server(&server.name) {
            if server.allocation.deployment.disk_retention == Retention::Temporary {
                self.get_backend().delete_server(backend_server.id);
            } else {
                // TODO: self.get_backend().stop_server(&backend_server);
            }
            self.inner.delete_server(backend_server.id);
        } else {
            error!(
                "{} to stop server {} because the server was {} by this driver",
                "Failed".red(),
                server.name,
                "never started".red()
            );
        }
    }
}

pub struct PterodactylNode {
    /* Cloud Identification */
    pub cloud_identifier: String,

    /* Informations about the node */
    pub backend: UnsafeCell<Option<Rc<Backend>>>,
    pub id: u32,
    pub name: String,
    pub capabilities: Capabilities,

    /* Dynamic Resources */
    pub allocations: Mutex<Vec<BAllocation>>,
    pub servers: Mutex<Vec<PanelServer>>,
}

impl PterodactylNode {
    fn get_allocations(&self) -> MutexGuard<Vec<BAllocation>> {
        // Safe as we are only run on the same thread
        self.allocations.lock().unwrap()
    }
    fn get_servers(&self) -> MutexGuard<Vec<PanelServer>> {
        // Safe as we are only run on the same thread
        self.servers.lock().unwrap()
    }

    fn find_allocation(&self, address: &Address) -> Option<BAllocation> {
        self.get_allocations()
            .iter()
            .find(|allocation| allocation.ip == address.ip && allocation.port == address.port)
            .cloned()
    }
    fn find_server(&self, name: &str) -> Option<PanelServer> {
        self.get_servers()
            .iter()
            .find(|server| server.name.name == name)
            .cloned()
    }
    fn delete_server(&self, id: u32) {
        self.get_servers().retain(|server| server.id != id);
    }
}
