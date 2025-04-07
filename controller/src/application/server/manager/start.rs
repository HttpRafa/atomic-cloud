use std::{collections::HashMap, mem::replace, sync::Arc};

use anyhow::Result;
use simplelog::{debug, error, info, warn};
use tokio::time::Instant;
use uuid::Uuid;

use crate::{
    application::{
        group::manager::GroupManager, node::manager::NodeManager, server::Server, Shared,
    },
    config::Config,
};

use super::{ServerManager, StartRequest, StartStage};

impl ServerManager {
    // Return true if the request should be ticked again.
    pub async fn handle_start_request(
        request: &mut StartRequest,
        servers: &mut HashMap<Uuid, Server>,
        config: &Config,
        nodes: &NodeManager,
        groups: &mut GroupManager,
        shared: &Arc<Shared>,
    ) -> Result<bool> {
        if request.nodes.is_empty() {
            warn!("Server {} has no nodes available to start on.", request.id);
            return Ok(false);
        }

        if let Some(when) = request.when {
            if when > Instant::now() {
                return Ok(true);
            }
        }

        // Cache old stage to compute the new stage based on the old stage
        let stage = replace(&mut request.stage, StartStage::Queued);
        request.stage = match stage {
            StartStage::Queued => {
                debug!("Allocating resources for server {}", request.id);
                match Self::allocate(0, request, nodes) {
                    Ok(handle) => StartStage::Allocating(0, handle),
                    Err(error) => {
                        warn!(
                            "Failed to allocate resources for server {}: {}",
                            request.id, error
                        );
                        return Ok(false);
                    }
                }
            }
            StartStage::Allocating(index, handle) => {
                if handle.is_finished() {
                    let ports = handle.await?;
                    if let Ok(ports) = ports {
                        if let Some(port) = ports.first() {
                            info!("Starting server {} listening on port {}", request.id, port);
                        }
                        match Self::start(
                            index, request, ports, servers, config, nodes, groups, shared,
                        )
                        .await
                        {
                            Ok(handle) => StartStage::Creating(handle),
                            Err(error) => {
                                warn!("Failed to create server {}: {}", request.id, error);
                                return Ok(false);
                            }
                        }
                    } else {
                        debug!(
                            "Driver failed to allocate resources for server {} on node {}",
                            request.id, request.nodes[index]
                        );
                        if index + 1 >= request.nodes.len() {
                            error!(
                                "No more nodes to try for server {}. Giving up...",
                                request.id
                            );
                            return Ok(false);
                        }
                        match Self::allocate(index + 1, request, nodes) {
                            Ok(handle) => StartStage::Allocating(index + 1, handle),
                            Err(error) => {
                                warn!(
                                    "Failed to allocate resources for server {}: {}",
                                    request.id, error
                                );
                                return Ok(false);
                            }
                        }
                    }
                } else {
                    StartStage::Allocating(index, handle)
                }
            }
            StartStage::Creating(handle) => {
                if handle.is_finished() {
                    // Register the screen with the shared screen manager
                    shared
                        .screens
                        .register_screen(request.id.uuid(), handle.await??)
                        .await;
                    debug!("Server {} has been started", request.id);
                    return Ok(false);
                }
                StartStage::Creating(handle)
            }
        };
        Ok(true)
    }
}
