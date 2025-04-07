use std::{collections::HashMap, mem::replace, sync::Arc};

use anyhow::Result;
use simplelog::{debug, info, warn};
use tokio::time::Instant;
use uuid::Uuid;

use crate::application::{
    cloudGroup::manager::GroupManager, node::manager::NodeManager, server::Server,
    user::manager::UserManager, Shared,
};

use super::{ServerManager, StopRequest, StopStage};

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
        let stage = replace(&mut request.stage, StopStage::Queued);
        request.stage = match stage {
            StopStage::Queued => {
                info!("Stopping server {}", request.server);
                match Self::stop(request, servers, nodes, shared).await {
                    Ok((handle, guard)) => StopStage::Running(handle, guard),
                    Err(error) => {
                        warn!("Failed to stop server {}: {}", request.server, error);
                        return Ok(false);
                    }
                }
            }
            StopStage::Running(handle, guard) => {
                if handle.is_finished() {
                    handle.await??;
                    debug!("Server {} is now stopping in the background waiting for the plugin to finish", request.server);
                    StopStage::Stopping(guard)
                } else {
                    StopStage::Running(handle, guard)
                }
            }
            StopStage::Stopping(guard) => {
                if guard.is_dropped() {
                    debug!(
                        "Server {} has been marked as stopped by the plugin. Freeing resources..",
                        request.server
                    );
                    debug!("Freeing resources for server {}", request.server);
                    match Self::free(request, servers, nodes) {
                        Ok(handle) => StopStage::Freeing(handle),
                        Err(error) => {
                            warn!("Failed to free server {}: {}", request.server, error);
                            return Ok(false);
                        }
                    }
                } else {
                    StopStage::Stopping(guard)
                }
            }
            StopStage::Freeing(handle) => {
                if handle.is_finished() {
                    handle.await??;
                    debug!(
                        "Server {} has been freed on the plugin side doing it on the controller...",
                        request.server
                    );
                    Self::remove(request, servers, groups, users, shared).await?;
                    debug!("Server {} has been stopped", request.server);
                    return Ok(false);
                }
                StopStage::Freeing(handle)
            }
        };
        Ok(true)
    }
}
