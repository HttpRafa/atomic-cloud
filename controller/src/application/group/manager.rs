use std::collections::HashMap;

use anyhow::Result;

use crate::application::TickService;

use super::Group;

pub struct GroupManager {
    groups: HashMap<String, Group>,
}

impl GroupManager {
    pub async fn init() -> Result<Self> {
        Ok(Self {
            groups: HashMap::new(),
        })
    }
}

impl TickService for GroupManager {
    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}