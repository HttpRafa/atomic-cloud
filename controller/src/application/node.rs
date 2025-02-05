use anyhow::Result;
use common::network::HostAndPort;
use getset::Getters;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use url::Url;

use super::{
    plugin::WrappedNode,
    server::{manager::StartRequest, Resources, Server, Spec},
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

    pub fn allocate(&self, request: &StartRequest) -> JoinHandle<Result<Vec<HostAndPort<String>>>> {
        self.instance.allocate(request)
    }
    pub fn free(&self, ports: &[HostAndPort]) -> JoinHandle<Result<()>> {
        self.instance.free(ports)
    }
    pub fn start(&self, server: &Server) -> JoinHandle<Result<()>> {
        self.instance.start(server)
    }
    pub fn restart(&self, server: &Server) -> JoinHandle<Result<()>> {
        self.instance.restart(server)
    }
    pub fn stop(&self, server: &Server) -> JoinHandle<Result<()>> {
        self.instance.stop(server)
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
    max_servers: Option<u32>,
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
