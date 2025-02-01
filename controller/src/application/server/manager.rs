use std::collections::HashMap;

use anyhow::Result;
use uuid::Uuid;

use crate::application::TickService;

use super::Server;

pub struct ServerManager {
    servers: HashMap<Uuid, Server>,
}

impl ServerManager {
    pub async fn init() -> Result<Self> {
        Ok(Self {
            servers: HashMap::new(),
        })
    }
}

impl TickService for ServerManager {
    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}