use std::collections::HashMap;

use anyhow::{anyhow, Result};
use common::network::HostAndPort;
use simplelog::{error, warn};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::{
    application::{
        auth::validator::WrappedAuthValidator,
        group::manager::GroupManager,
        node::{manager::NodeManager, Allocation},
        server::{Flags, Health, Server, State},
    },
    config::Config,
};

use super::{RestartRequest, ServerManager, StartRequest, StopRequest};

impl ServerManager {
    pub fn allocate(
        index: usize,
        request: &StartRequest,
        nodes: &NodeManager,
    ) -> Result<JoinHandle<Result<Vec<HostAndPort>>>> {
        if let Some(name) = request.nodes.get(index) {
            let node = nodes.get_node(name);
            if let Some(node) = node {
                Ok(node.allocate(request))
            } else {
                Err(anyhow!(
                    "Node {} not found while trying to allocate ports for server {}",
                    name,
                    request.name
                ))
            }
        } else {
            Err(anyhow!(
                "Index of node in request is out of bounds. Has someone tampered with the request?"
            ))
        }
    }
    pub async fn start(
        index: usize,
        request: &StartRequest,
        ports: Vec<HostAndPort>,
        servers: &mut HashMap<Uuid, Server>,
        config: &Config,
        nodes: &NodeManager,
        groups: &mut GroupManager,
        validator: &WrappedAuthValidator,
    ) -> Result<JoinHandle<Result<()>>> {
        if let Some(name) = request.nodes.get(index) {
            let node = nodes.get_node(name);
            if let Some(node) = node {
                let mut server = Server {
                    uuid: request.uuid,
                    name: request.name.clone(),
                    group: request.group.clone(),
                    node: name.clone(),
                    allocation: Allocation {
                        ports,
                        resources: request.resources.clone(),
                        spec: request.spec.clone(),
                    },
                    connected_users: 0,
                    token: validator.register_server(request.uuid).await,
                    health: Health::new(*config.startup_timeout(), *config.heartbeat_timeout()),
                    state: State::Starting,
                    flags: Flags::default(),
                };
                let handle = node.start(&server);
                if let Some(group) = &server.group {
                    if let Some(group) = groups.get_group_mut(group) {
                        group.set_server_active(&server.uuid);
                    } else {
                        warn!("Group {} not found while trying to start server {}. Removing group from server", group, request.name);
                        server.group = None;
                    }
                }
                servers.insert(server.uuid, server);
                Ok(handle)
            } else {
                Err(anyhow!(
                    "Node {} not found while trying to allocate ports for server {}",
                    name,
                    request.name
                ))
            }
        } else {
            Err(anyhow!(
                "Index of node in request is out of bounds. Has someone tampered with the request?"
            ))
        }
    }
    pub fn restart(
        request: &RestartRequest,
        servers: &mut HashMap<Uuid, Server>,
        config: &Config,
        nodes: &NodeManager,
    ) -> Result<JoinHandle<Result<()>>> {
        if let Some(server) = servers.get_mut(&request.server) {
            if let Some(node) = nodes.get_node(&server.node) {
                server.state = State::Restarting;
                server.health = Health::new(*config.startup_timeout(), *config.heartbeat_timeout());
                Ok(node.restart(server))
            } else {
                Err(anyhow!(
                    "Node {} not found while trying to restart {}",
                    server.node,
                    request.server
                ))
            }
        } else {
            Err(anyhow!(
                "Server {} not found while trying to restart",
                request.server
            ))
        }
    }
    pub fn free(
        request: &StopRequest,
        servers: &mut HashMap<Uuid, Server>,
        nodes: &NodeManager,
    ) -> Result<JoinHandle<Result<()>>> {
        if let Some(server) = servers.get(&request.server) {
            if let Some(node) = nodes.get_node(&server.node) {
                Ok(node.free(&server.allocation.ports))
            } else {
                Err(anyhow!(
                    "Node {} not found while trying to free resources {}",
                    server.node,
                    request.server
                ))
            }
        } else {
            Err(anyhow!(
                "Server {} not found while trying to free resources",
                request.server
            ))
        }
    }
    pub async fn stop(
        request: &StopRequest,
        servers: &mut HashMap<Uuid, Server>,
        nodes: &NodeManager,
        groups: &mut GroupManager,
        validator: &WrappedAuthValidator,
    ) -> Result<JoinHandle<Result<()>>> {
        if let Some(server) = servers.get_mut(&request.server) {
            if let Some(node) = nodes.get_node(&server.node) {
                if let Some(group) = &server.group {
                    if let Some(group) = groups.get_group_mut(group) {
                        group.remove_server(&server.uuid);
                    } else {
                        error!("Group {} not found while trying to stop server {}. Removing group from server", group, server.name);
                        server.group = None;
                    }
                }
                validator.unregister(&server.token).await;

                // TODO: Cleanup connected users
                // TODO: Cleanup subscriptions

                Ok(node.stop(server))
            } else {
                Err(anyhow!(
                    "Node {} not found while trying to stop {}",
                    server.node,
                    request.server
                ))
            }
        } else {
            Err(anyhow!(
                "Server {} not found while trying to stop",
                request.server
            ))
        }
    }
}
