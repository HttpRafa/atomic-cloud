use anyhow::Result;
use common::{config::SaveToTomlFile, network::HostAndPort};
use getset::Getters;
use manager::stored::StoredNode;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tonic::Status;
use url::Url;

use crate::storage::Storage;

use super::{
    group::manager::GroupManager, plugin::BoxedNode, server::{manager::{ServerManager, StartRequest}, Resources, Server, Spec}
};

pub mod manager;

#[derive(Getters)]
pub struct Node {
    /* Plugin */
    plugin: String,
    instance: BoxedNode,

    /* Settings */
    #[getset(get = "pub")]
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

    pub fn set_active(&mut self, active: bool, servers: &ServerManager, groups: &GroupManager) -> Result<(), SetActiveError>{
        if active && self.status == LifecycleStatus::Inactive {
            // Activate node

            self.status = LifecycleStatus::Active;
            self.save().map_err(|error| SetActiveError::Error(error))?;
        } else if !active && self.status == LifecycleStatus::Active {
            // Retire node
            if groups.is_node_used(&self.name) {
                return Err(SetActiveError::NodeInUseByGroup);
            }
            if servers.is_node_used(&self.name) {
                return Err(SetActiveError::NodeInUseByServer);
            }

            self.status = LifecycleStatus::Inactive;
            self.save().map_err(|error| SetActiveError::Error(error))?;
        }

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

    pub fn save(&self) -> Result<()> {
        StoredNode::from(self).save(&Storage::node_file(&self.name), true)
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

pub enum SetActiveError {
    NodeInUseByGroup,
    NodeInUseByServer,
    Error(anyhow::Error),
}

impl Allocation {
    pub fn primary_port(&self) -> Option<&HostAndPort> {
        self.ports.first()
    }
}

impl From<SetActiveError> for Status {
    fn from(val: SetActiveError) -> Self {
        match val {
            SetActiveError::NodeInUseByGroup => Status::unavailable("Node in use by some group"),
            SetActiveError::NodeInUseByServer => Status::not_found("Node in use by some server"),
            SetActiveError::Error(error) => Status::internal(format!("Error: {}", error)),
        }
    }
}
