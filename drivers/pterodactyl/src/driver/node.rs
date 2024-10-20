use colored::Colorize;
use server::{PanelServer, ServerName};
use std::{
    cell::UnsafeCell,
    rc::Rc,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
    vec,
};

use crate::{
    error,
    exports::node::driver::bridge::{
        Address, Capabilities, GuestGenericNode, RemoteController, Retention, Server,
        ServerProposal,
    },
    info, warn,
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
        controller: RemoteController,
    ) -> Self {
        Self {
            inner: Rc::new(PterodactylNode {
                cloud_identifier,
                backend: UnsafeCell::new(None),
                id: id.unwrap(),
                name,
                capabilities,
                controller,
                allocations: RwLock::new(vec![]),
                servers: RwLock::new(vec![]),
            }),
        }
    }

    /* This method expects that the Pterodactyl Allocations are only accessed by one atomic cloud instance */
    fn allocate_addresses(&self, server: ServerProposal) -> Result<Vec<Address>, String> {
        let amount = server.resources.addresses;

        if server.deployment.disk_retention == Retention::Permanent {
            let name = ServerName::new(
                &self.inner.cloud_identifier,
                &server.name,
                &Retention::Permanent,
            );

            // Check if a server with the same name is already exists
            if let Some(backend_server) = self.get_backend().get_server_by_name(&name) {
                // Get the allocations that are already used by this server
                let mut allocations = self
                    .get_backend()
                    .get_allocations_by_server(&backend_server.identifier);

                if (allocations.1.len() + 1) as u32 != server.resources.addresses {
                    warn!("The server {} has a different amount of addresses than the panel has allocated. This may cause issues.", server.name);
                    // TODO: Add a way to fix this
                }

                allocations.1.insert(0, allocations.0); // Add primary allocation to the list
                return Ok(allocations.1.into_iter().map(|x| x.into()).collect());
            }
        }

        let mut used = self.inner.get_allocations_mut();
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
        self.inner.get_allocations_mut().retain(|x| {
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

        // Check if a server with the same name is already exists
        if let Some(backend_server) = self.get_backend().get_server_by_name(&name) {
            if deployment.disk_retention == Retention::Temporary {
                error!(
                    "Server {} already exists on the panel, but the disk retention is temporary",
                    server.name.blue()
                );
                return;
            }
            // Just use the existing server and change its settings
            info!(
                "Server {} already exists on the panel, updating settings and starting...",
                server.name.blue()
            );
            self.get_backend()
                .update_settings(self, allocations[0].id, &backend_server, &server);
            self.get_backend().start_server(&backend_server.identifier);
            self.inner.get_servers_mut().push(PanelServer::new(
                backend_server.id,
                backend_server.identifier,
                name,
            ));
        } else {
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
                &server,
                self,
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
                info!(
                    "Server {} successfully {} on the panel",
                    server.name.blue(),
                    "created".green()
                );
                self.inner.get_servers_mut().push(PanelServer::new(
                    server.id,
                    server.identifier,
                    name,
                ));
            }
        }
    }

    fn restart_server(&self, server: Server) {
        if let Some(backend_server) = self.inner.find_server(&server.name) {
            self.get_backend().restart_server(&backend_server);
            info!(
                "Panel is {} the server {}...",
                "restarting".yellow(),
                backend_server.name.generate().blue(),
            );
        } else {
            error!(
                "{} to restart server {} because the server was {} by this driver",
                "Failed".red(),
                server.name,
                "never started".red()
            );
        }
    }

    fn stop_server(&self, server: Server) {
        if let Some(backend_server) = self.inner.find_server(&server.name) {
            if server.allocation.deployment.disk_retention == Retention::Temporary {
                self.get_backend().delete_server(backend_server.id);
                info!(
                    "Server {} successfully {} from the panel",
                    backend_server.name.generate().blue(),
                    "deleted".red()
                );
            } else {
                self.get_backend().stop_server(&backend_server);
                info!(
                    "Panel is {} the server {}...",
                    "stopping".red(),
                    backend_server.name.generate().blue(),
                );
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
    pub controller: RemoteController,

    /* Dynamic Resources */
    pub allocations: RwLock<Vec<BAllocation>>,
    pub servers: RwLock<Vec<PanelServer>>,
}

impl PterodactylNode {
    fn get_allocations(&self) -> RwLockReadGuard<Vec<BAllocation>> {
        // Safe as we are only run on the same thread
        self.allocations.read().unwrap()
    }
    fn get_allocations_mut(&self) -> RwLockWriteGuard<Vec<BAllocation>> {
        // Safe as we are only run on the same thread
        self.allocations.write().unwrap()
    }
    fn get_servers(&self) -> RwLockReadGuard<Vec<PanelServer>> {
        // Safe as we are only run on the same thread
        self.servers.read().unwrap()
    }
    fn get_servers_mut(&self) -> RwLockWriteGuard<Vec<PanelServer>> {
        // Safe as we are only run on the same thread
        self.servers.write().unwrap()
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
        self.get_servers_mut().retain(|server| server.id != id);
    }
}
