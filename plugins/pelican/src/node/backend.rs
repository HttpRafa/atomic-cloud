
use anyhow::{anyhow, Result};
use url::Url;

use crate::plugin::config::Config;

pub mod allocation;
mod common;
pub mod node;
pub mod server;
pub mod user;

pub struct Backend {
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

impl Backend {
    pub fn new(config: &Config, node: &str) -> Result<Self> {
        let mut backend = Self {
            url: config.url().clone(),
            token: config.token().to_string(),
            username: config.username().to_string(),
            user_token: config.user_token().to_string(),

            node_id: 0,
            user_id: 0,
        };

        // Update the node_id field
        backend.node_id = backend
            .get_node_by_name(node)
            .ok_or(anyhow!(
                "Failed to get node {} from panel. Does it exist?",
                config.username()
            ))?
            .id;

        // Update the user_id field
        backend.user_id = backend
            .get_user_by_name(&backend.username)
            .ok_or(anyhow!(
                "Failed to get user {} from panel. Does he exist?",
                config.username()
            ))?
            .id;

        Ok(backend)
    }
}
