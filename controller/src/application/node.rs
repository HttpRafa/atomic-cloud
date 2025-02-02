use anyhow::Result;
use common::network::HostAndPort;
use getset::Getters;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{
    plugin::WrappedNode,
    server::{Resources, Spec},
};

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

#[derive(Getters)]
pub struct Allocation {
    #[getset(get = "pub")]
    pub ports: Vec<HostAndPort>,
    #[getset(get = "pub")]
    pub resources: Resources,
    #[getset(get = "pub")]
    pub spec: Spec,
}

#[derive(Serialize, Deserialize, Clone, Default, Getters)]
pub struct Capabilities {
    #[getset(get = "pub")]
    memory: Option<u32>,
    #[getset(get = "pub")]
    max_allocations: Option<u32>,
    #[getset(get = "pub")]
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

#[derive(Serialize, Deserialize, Clone, Getters)]
pub struct RemoteController {
    #[getset(get = "pub")]
    address: Url,
}
