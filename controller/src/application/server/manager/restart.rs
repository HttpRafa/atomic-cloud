use std::{collections::HashMap, mem::replace};

use anyhow::Result;
use simplelog::{debug, warn};
use tokio::time::Instant;
use uuid::Uuid;

use crate::{
    application::{node::manager::NodeManager, server::Server},
    config::Config,
};

use super::{ActionStage, RestartRequest, ServerManager};

impl ServerManager {
    // Return true if the request should be ticked again.
    pub async fn handle_restart_request(
        request: &mut RestartRequest,
        servers: &mut HashMap<Uuid, Server>,
        config: &Config,
        nodes: &NodeManager,
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
                debug!("Restarting server {}", request.server);
                match Self::restart(request, servers, config, nodes) {
                    Ok(handle) => ActionStage::Running(handle),
                    Err(error) => {
                        warn!("Failed to restart server {}: {}", request.server, error);
                        return Ok(false);
                    }
                }
            }
            ActionStage::Running(handle) => {
                if handle.is_finished() {
                    handle.await??;
                    debug!("Server {} has been restarted", request.server);
                    return Ok(false);
                }
                ActionStage::Running(handle)
            }
            ActionStage::Freeing(_) => {
                warn!("Server {} is in an invalid state", request.server);
                return Ok(false);
            }
        };
        Ok(true)
    }
}
