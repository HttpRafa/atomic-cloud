use std::collections::HashMap;

use anyhow::{Result};

use crate::application::TickService;

use super::Node;

pub struct NodeManager {
    nodes: HashMap<String, Node>,
}

impl NodeManager {
    pub async fn init() -> Result<Self> {
        Ok(Self {
            nodes: HashMap::new(),
        })
    }
}

impl TickService for NodeManager {
    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}