use anyhow::Result;
use common::allocator::NumberAllocator;
use serde::{Deserialize, Serialize};
use simplelog::debug;
use uuid::Uuid;

use crate::{application::server::manager::StopRequest, config::Config};

use super::{
    node::LifecycleStatus,
    server::{
        manager::{ServerManager, StartRequest},
        Resources, Spec,
    },
};

pub mod manager;

pub struct Group {
    /* Settings */
    name: String,
    status: LifecycleStatus,

    /* Where? */
    nodes: Vec<String>,
    constraints: StartConstraints,
    scaling: ScalingPolicy,

    /* How? */
    resources: Resources,
    spec: Spec,

    /* What do i need to know? */
    id_allocator: NumberAllocator<usize>,
    servers: Vec<AssociatedServer>,
}

impl Group {
    pub fn tick(&mut self, config: &Config, servers: &mut ServerManager) -> Result<()> {
        if self.status == LifecycleStatus::Inactive {
            // Do not tick this group because it is inactive
            return Ok(());
        }

        let mut target_count = self.constraints.minimum;

        // Apply scling policy
        if self.scaling.enabled {
            self.servers.retain(|server| match server {
                AssociatedServer::Active(server) => {
                    servers.get_server(server).is_some_and(|server| {
                        if *server.connected_users() as f32 / *self.spec.max_players() as f32
                            >= self.scaling.start_threshold
                        {
                            target_count += 1;
                        }
                        true
                    })
                }
                _ => true,
            });

            if self.scaling.stop_empty_servers && self.servers.len() as u32 > target_count {
                let mut to_stop = self.servers.len() as u32 - target_count;
                let mut requests = vec![];
                self.servers.retain(|server| match server {
                    AssociatedServer::Active(server) => {
                        servers.get_server_mut(server).is_some_and(|server| {
                            if server.connected_users() == &0 {
                                if server.flags().should_stop() && to_stop > 0 {
                                    debug!(
                                    "Server {} is empty and reached the timeout, stopping it...",
                                    server.id()
                                );
                                    requests.push(StopRequest::new(None, server.id()));
                                    to_stop -= 1;
                                } else {
                                    debug!(
                                        "Server {} is empty, starting stop timer...",
                                        server.id()
                                    );
                                    server
                                        .flags_mut()
                                        .replace_stop(*config.empty_server_timeout());
                                }
                            } else if server.flags().is_stop_set() {
                                debug!(
                                    "Server {} is no longer empty, clearing stop timer...",
                                    server.id()
                                );
                                server.flags_mut().clear_stop();
                            }
                            true
                        })
                    }
                    _ => true,
                });
                servers.schedule_stops(requests);
            }
        }

        for requested in 0..(target_count as usize).saturating_sub(self.servers.len()) {
            if (self.servers.len() + requested) >= target_count as usize {
                break;
            }

            let id = self
                .id_allocator
                .allocate()
                .expect("We reached the maximum unit count. Wow this is a lot of units");
            let request = StartRequest::new(
                None,
                self.constraints.priority,
                format!("{}-{}", self.name, id),
                Some(self.name.clone()),
                &self.nodes,
                &self.resources,
                &self.spec,
            );
            self.servers
                .push(AssociatedServer::Queueing(*request.id().uuid()));
            debug!(
                "Scheduled server({}) start for group {}",
                request.id(),
                self.name
            );
            servers.schedule_start(request);
        }

        Ok(())
    }

    pub fn set_server_active(&mut self, server_uuid: &Uuid) {
        self.servers.retain(|server| {
            if let AssociatedServer::Queueing(uuid) = server {
                if uuid == server_uuid {
                    return false;
                }
            }
            true
        });
        self.servers.push(AssociatedServer::Active(*server_uuid));
    }

    pub fn remove_server(&mut self, server_uuid: &Uuid) {
        self.servers.retain(|server| {
            if let AssociatedServer::Active(uuid) = server {
                if uuid == server_uuid {
                    return false;
                }
            }
            true
        });
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct StartConstraints {
    minimum: u32,
    maximum: u32,
    priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ScalingPolicy {
    enabled: bool,
    start_threshold: f32,
    stop_empty_servers: bool,
}

pub enum AssociatedServer {
    Queueing(Uuid),
    Active(Uuid),
}
