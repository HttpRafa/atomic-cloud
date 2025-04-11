use std::collections::HashMap;

use anyhow::{anyhow, Result};
use common::allocator::NumberAllocator;
use getset::{Getters, Setters};
use manager::stored::StoredGroup;
use serde::{Deserialize, Serialize};
use simplelog::{debug, info};
use tokio::fs;

use crate::{
    application::server::manager::StopRequest,
    config::Config,
    resource::DeleteResourceError,
    storage::{SaveToTomlFile, Storage},
};

use super::{
    node::LifecycleStatus,
    server::{
        manager::{ServerManager, StartRequest},
        NameAndUuid, Resources, Server, Spec,
    },
};

pub mod manager;

#[derive(Getters, Setters)]
pub struct Group {
    /* Settings */
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    status: LifecycleStatus,

    /* Where? */
    #[getset(get = "pub", set = "pub")]
    nodes: Vec<String>,
    #[getset(get = "pub", set = "pub")]
    constraints: StartConstraints,
    #[getset(get = "pub", set = "pub")]
    scaling: ScalingPolicy,

    /* How? */
    #[getset(get = "pub", set = "pub")]
    resources: Resources,
    #[getset(get = "pub", set = "pub")]
    spec: Spec,

    /* What do i need to know? */
    id_allocator: NumberAllocator<usize>,
    servers: HashMap<NameAndUuid, GroupServer>,
}

impl Group {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "I have to clean this up"
    )]
    pub fn tick(&mut self, config: &Config, servers: &mut ServerManager) -> Result<()> {
        if self.status == LifecycleStatus::Inactive {
            // Do not tick this group because it is inactive
            return Ok(());
        }

        let mut target_count = self.constraints.minimum;

        // Apply scaling policy
        if self.scaling.enabled {
            self.servers.retain(|id, server| match &server.1 {
                Stage::Active => servers.get_server(id.uuid()).is_some_and(|server| {
                    if *server.connected_users() as f32 / *self.spec.max_players() as f32
                        >= self.scaling.start_threshold
                    {
                        target_count += 1;
                    }
                    true
                }),
                Stage::Queueing => true,
            });

            if self.scaling.stop_empty_servers && self.servers.len() as u32 > target_count {
                let mut to_stop = self.servers.len() as u32 - target_count;
                let mut requests = vec![];
                self.servers.retain(|id, server| match &server.1 {
                    Stage::Active => servers.get_server_mut(id.uuid()).is_some_and(|server| {
                        if server.connected_users() == &0 {
                            if server.flags().should_stop() && to_stop > 0 {
                                debug!(
                                    "Server {} is empty and reached the timeout, stopping it...",
                                    server.id()
                                );
                                requests.push(StopRequest::new(None, server.id().clone()));
                                to_stop -= 1;
                                server.flags_mut().clear_stop();
                            } else if !server.flags().is_stop_set() {
                                debug!("Server {} is empty, starting stop timer...", server.id());
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
                    }),
                    Stage::Queueing => true,
                });
                servers.schedule_stops(requests);
            }
        }

        for requested in 0..(target_count as usize).saturating_sub(self.servers.len()) {
            if (self.servers.len() + requested) >= target_count as usize {
                break;
            }

            let id = self.id_allocator.allocate().ok_or(anyhow!(
                "We reached the maximum server count. Wow this is a lot of servers"
            ))?;
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
                .insert(request.id().clone(), GroupServer::new(id));
            debug!(
                "Scheduled server({}) start for group {}",
                request.id(),
                self.name
            );
            servers.schedule_start(request);
        }

        Ok(())
    }

    pub async fn delete(&mut self) -> Result<(), DeleteResourceError> {
        if self.status == LifecycleStatus::Active {
            return Err(DeleteResourceError::StillActive);
        }
        if !self.servers.is_empty() {
            return Err(DeleteResourceError::StillInUse);
        }
        let path = Storage::group_file(&self.name);
        if path.exists() {
            fs::remove_file(path)
                .await
                .map_err(|error| DeleteResourceError::Error(error.into()))?;
        }

        Ok(())
    }

    pub async fn set_active(&mut self, active: bool, servers: &mut ServerManager) -> Result<()> {
        if active && self.status == LifecycleStatus::Inactive {
            // Activate group

            self.status = LifecycleStatus::Active;
            self.save().await?;
            info!("Group {} is now active", self.name);
        } else if !active && self.status == LifecycleStatus::Active {
            // Retire group
            // Stop all servers and cancel all starts
            self.servers.retain(|id, server| match &server.1 {
                Stage::Active => {
                    servers.schedule_stop(StopRequest::new(None, id.clone()));
                    true
                }
                Stage::Queueing => {
                    servers.cancel_start(id.uuid());
                    false
                }
            });

            self.status = LifecycleStatus::Inactive;
            self.save().await?;
            info!("Group {} is now inactive", self.name);
        }

        Ok(())
    }

    pub fn find_free_server<'a>(&self, servers: &'a ServerManager) -> Option<&'a Server> {
        self.servers
            .iter()
            .find_map(|(id, server)| match &server.1 {
                Stage::Active => servers.get_server(id.uuid()),
                Stage::Queueing => None,
            })
    }

    pub fn set_server_active(&mut self, id: &NameAndUuid) {
        self.servers.entry(id.clone()).and_modify(|server| {
            server.1 = Stage::Active;
        });
    }

    pub fn remove_server(&mut self, id: &NameAndUuid) {
        if let Some(server) = self.servers.remove(id) {
            self.id_allocator.release(server.0);
        }
    }

    pub async fn save(&self) -> Result<()> {
        StoredGroup::from(self)
            .save(&Storage::group_file(&self.name), true)
            .await
    }
}

#[derive(Serialize, Deserialize, Clone, Getters)]
pub struct StartConstraints {
    #[getset(get = "pub")]
    minimum: u32,
    #[getset(get = "pub")]
    maximum: u32,
    #[getset(get = "pub")]
    priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Default, Getters)]
pub struct ScalingPolicy {
    #[getset(get = "pub")]
    enabled: bool,
    #[getset(get = "pub")]
    start_threshold: f32,
    #[getset(get = "pub")]
    stop_empty_servers: bool,
}

struct GroupServer(usize, Stage);

enum Stage {
    Queueing,
    Active,
}

impl GroupServer {
    pub fn new(id: usize) -> Self {
        Self(id, Stage::Queueing)
    }
}

impl StartConstraints {
    pub fn new(minimum: u32, maximum: u32, priority: i32) -> Self {
        Self {
            minimum,
            maximum,
            priority,
        }
    }
}

impl ScalingPolicy {
    pub fn new(enabled: bool, start_threshold: f32, stop_empty_servers: bool) -> Self {
        Self {
            enabled,
            start_threshold,
            stop_empty_servers,
        }
    }
}
