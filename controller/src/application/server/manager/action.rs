use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use common::network::HostAndPort;
use simplelog::{error, warn};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::{
    application::{
        group::manager::GroupManager,
        node::{manager::NodeManager, Allocation},
        server::{screen::BoxedScreen, Flags, Heart, Server, State},
        user::manager::UserManager,
        Shared,
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
                    request.id
                ))
            }
        } else {
            Err(anyhow!(
                "Index of node in request is out of bounds. Has someone tampered with the request?"
            ))
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn start(
        index: usize,
        request: &StartRequest,
        ports: Vec<HostAndPort>,
        servers: &mut HashMap<Uuid, Server>,
        config: &Config,
        nodes: &NodeManager,
        groups: &mut GroupManager,
        shared: &Arc<Shared>,
    ) -> Result<JoinHandle<Result<BoxedScreen>>> {
        if let Some(name) = request.nodes.get(index) {
            let node = nodes.get_node(name);
            if let Some(node) = node {
                let mut server = Server {
                    id: request.id.clone(),
                    group: request.group.clone(),
                    node: name.clone(),
                    allocation: Allocation {
                        ports,
                        resources: request.resources.clone(),
                        spec: request.spec.clone(),
                    },
                    connected_users: 0,
                    token: shared.auth.register_server(request.id.uuid).await,
                    heart: Heart::new(*config.startup_timeout(), *config.heartbeat_timeout()),
                    state: State::Starting,
                    flags: Flags::default(),
                    ready: false,
                };
                let handle = node.start(&server);
                if let Some(group) = &server.group {
                    if let Some(group) = groups.get_group_mut(group) {
                        group.set_server_active(&server.id);
                    } else {
                        warn!("Group {} not found while trying to start server {}. Removing group from server", group, request.id);
                        server.group = None;
                    }
                }
                servers.insert(server.id.uuid, server);
                Ok(handle)
            } else {
                Err(anyhow!(
                    "Node {} not found while trying to allocate ports for server {}",
                    name,
                    request.id
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
        if let Some(server) = servers.get_mut(request.server.uuid()) {
            if let Some(node) = nodes.get_node(&server.node) {
                server.state = State::Restarting;
                server.heart = Heart::new(*config.startup_timeout(), *config.heartbeat_timeout());
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
        if let Some(server) = servers.get_mut(request.server.uuid()) {
            server.state = State::Stopping;
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
        users: &mut UserManager,
        shared: &Arc<Shared>,
    ) -> Result<JoinHandle<Result<()>>> {
        if let Some(server) = servers.get_mut(request.server.uuid()) {
            if let Some(node) = nodes.get_node(&server.node) {
                if let Some(group) = &server.group {
                    if let Some(group) = groups.get_group_mut(group) {
                        group.remove_server(&server.id);
                    } else {
                        error!("Group {} not found while trying to stop server {}. Removing group from server", group, server.id);
                        server.group = None;
                    }
                }
                shared.auth.unregister(&server.token).await;

                users.remove_users_on_server(server.id.uuid());

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
