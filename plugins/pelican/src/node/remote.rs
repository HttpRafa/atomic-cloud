
use anyhow::{anyhow, Result};
use url::Url;

use crate::plugin::config::Config;

mod common;
mod node;
mod user;

pub struct Remote {
    url: Url,
    token: String,
    username: String,
    user_token: String,

    node_id: u32,
    user_id: u32,
}

pub enum Endpoint {
    Client,
    Application,
}

impl Remote {
    pub fn new(config: &Config, node: &str) -> Result<Self> {
        let mut remote = Self {
            url: config.url().clone(),
            token: config.token().to_string(),
            username: config.username().to_string(),
            user_token: config.user_token().to_string(),

            node_id: 0,
            user_id: 0,
        };

        // Update the node_id field
        remote.node_id = remote
            .get_node_by_name(node)
            .ok_or(anyhow!(
                "Failed to get node {} from panel. Does it exist?",
                config.username()
            ))?
            .id;

        // Update the user_id field
        remote.user_id = remote
            .get_user_by_name(&remote.username)
            .ok_or(anyhow!(
                "Failed to get user {} from panel. Does he exist?",
                config.username()
            ))?
            .id;

        Ok(remote)
    }
}
