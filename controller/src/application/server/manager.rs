use std::collections::HashMap;

use anyhow::Result;
use uuid::Uuid;

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

    pub fn get_server(&self, uuid: &Uuid) -> Option<&Server> {
        self.servers.get(uuid)
    }
}

// Ticking
impl ServerManager {
    pub async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
