use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use url::Url;

use super::plugin::WrappedNode;

pub mod manager;

pub struct Node {
    /* Plugin */
    plugin: String,
    instance: WrappedNode,

    /* Settings */
    name: String,
    capabilities: Capabilities,
    status: LifecycleStatus,

    /* Controller */
    controller: RemoteController,
}

impl Node {
    pub fn tick(&self) -> Result<()> {
        if self.status == LifecycleStatus::Inactive {
            // Do not tick this node because it is inactive
            return Ok(());
        }

        self.instance.tick();
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Capabilities {
    memory: Option<u32>,
    max_allocations: Option<u32>,
    child: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub enum LifecycleStatus {
    #[serde(rename = "inactive")]
    #[default]
    Inactive,
    #[serde(rename = "active")]
    Active,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoteController {
    address: Url,
}

impl Capabilities {
    pub fn get_memory(&self) -> Option<u32> {
        self.memory
    }
    pub fn get_max_allocations(&self) -> Option<u32> {
        self.max_allocations
    }
    pub fn get_child(&self) -> Option<&str> {
        self.child.as_deref()
    }
}

impl RemoteController {
    pub fn get_address(&self) -> &Url {
        &self.address
    }
}
