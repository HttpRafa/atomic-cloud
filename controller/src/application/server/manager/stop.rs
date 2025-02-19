use std::{collections::HashMap, mem::replace, sync::Arc};

use anyhow::Result;
use simplelog::{debug, warn};
use tokio::time::Instant;
use uuid::Uuid;

use crate::application::{
    group::manager::GroupManager, node::manager::NodeManager, server::Server,
    user::manager::UserManager, Shared,
};

use super::{ActionStage, ServerManager, StopRequest};

impl ServerManager {
    // Return true if the request should be ticked again.
    pub async fn handle_stop_request(
        request: &mut StopRequest,
        servers: &mut HashMap<Uuid, Server>,
        nodes: &NodeManager,
        groups: &mut GroupManager,
        users: &mut UserManager,
        shared: &Arc<Shared>,
    ) -> Result<bool> {
        if let Some(when) = request.when {
            if when > Instant::now() {
                return Ok(true);
            }
        }

        // Cache old stage to compute the new stage based on the old stage
        let stage = replace(&mut request.stage, ActionStage::Queued);
        request.stage = match stage {
            ActionStage::Queued => {
                debug!("Freeing resources for server {}", request.server);
                match Self::free(request, servers, nodes) {
                    Ok(handle) => ActionStage::Freeing(handle),
                    Err(error) => {
                        warn!("Failed to free server {}: {}", request.server, error);
                        return Ok(false);
                    }
                }
            }
            ActionStage::Freeing(handle) => {
                if handle.is_finished() {
                    handle.await??;
                    debug!("Stopping server {}", request.server);
                    match Self::stop(request, servers, nodes, groups, users, shared).await {
                        Ok(handle) => ActionStage::Running(handle),
                        Err(error) => {
                            warn!("Failed to stop server {}: {}", request.server, error);
                            return Ok(false);
                        }
                    }
                } else {
                    ActionStage::Freeing(handle)
                }
            }
            ActionStage::Running(handle) => {
                if handle.is_finished() {
                    handle.await??;
                    // Remove the screen from the shared screen manager
                    shared
                        .screens
                        .unregister_screen(request.server.uuid())
                        .await?;
                    debug!("Server {} has been stopped", request.server);
                    return Ok(false);
                }
                ActionStage::Running(handle)
            }
        };
        Ok(true)
    }
}
