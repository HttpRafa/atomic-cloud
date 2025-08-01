use anyhow::Result;
use common::network::HostAndPort;
use getset::{Getters, Setters};
use manager::stored::StoredNode;
use serde::{Deserialize, Serialize};
use simplelog::info;
use tokio::{fs, task::JoinHandle};
use tonic::Status;
use url::Url;

use crate::{
    resource::DeleteResourceError,
    storage::{SaveToTomlFile, Storage},
};

use super::{
    plugin::BoxedNode,
    server::{
        Resources, Server, Specification, guard::Guard, manager::StartRequest, screen::BoxedScreen,
    },
};

pub mod manager;

#[derive(Getters, Setters)]
pub struct Node {
    /* Plugin */
    #[getset(get = "pub")]
    plugin: String,
    instance: BoxedNode,

    /* Settings */
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub", set = "pub")]
    capabilities: Capabilities,
    #[getset(get = "pub")]
    status: LifecycleStatus,

    /* Controller */
    #[getset(get = "pub", set = "pub")]
    controller: Url,
}

impl Node {
    #[allow(clippy::unnecessary_wraps)]
    pub fn tick(&self) -> Result<()> {
        // Always tick the node in the plugin
        self.instance.tick();

        if self.status == LifecycleStatus::Inactive {
            // Do not tick this node because it is inactive
            return Ok(());
        }

        Ok(())
    }

    pub async fn delete(&mut self) -> Result<(), DeleteResourceError> {
        if self.status == LifecycleStatus::Active {
            return Err(DeleteResourceError::StillActive);
        }
        let path = Storage::group_file(&self.name);
        if path.exists() {
            fs::remove_file(path)
                .await
                .map_err(|error| DeleteResourceError::Error(error.into()))?;
        }

        Ok(())
    }

    pub async fn set_active(&mut self, active: bool) -> Result<()> {
        if active && self.status == LifecycleStatus::Inactive {
            // Activate node

            self.status = LifecycleStatus::Active;
            self.save().await?;
            info!("Node {} is now active", self.name);
        } else if !active && self.status == LifecycleStatus::Active {
            // Retire node

            self.status = LifecycleStatus::Inactive;
            self.save().await?;
            info!("Node {} is now inactive", self.name);
        }

        Ok(())
    }

    pub fn allocate(&self, request: &StartRequest) -> JoinHandle<Result<Vec<HostAndPort<String>>>> {
        self.instance.allocate(request)
    }
    pub fn free(&self, ports: &[HostAndPort]) -> JoinHandle<Result<()>> {
        self.instance.free(ports)
    }
    pub fn start(&self, server: &Server) -> JoinHandle<Result<BoxedScreen>> {
        self.instance.start(server)
    }
    pub fn restart(&self, server: &Server) -> JoinHandle<Result<()>> {
        self.instance.restart(server)
    }
    pub fn stop(&self, server: &Server, guard: Guard) -> JoinHandle<Result<()>> {
        self.instance.stop(server, guard)
    }

    pub async fn save(&self) -> Result<()> {
        StoredNode::from(self)
            .save(&Storage::node_file(&self.name), true)
            .await
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        self.instance.cleanup().await
    }
}

#[derive(Getters, Clone)]
pub struct Allocation {
    #[getset(get = "pub")]
    pub ports: Vec<HostAndPort>,
    #[getset(get = "pub")]
    pub resources: Resources,
    #[getset(get = "pub")]
    pub specification: Specification,
}

#[derive(Serialize, Deserialize, Clone, Default, Getters)]
pub struct Capabilities {
    #[getset(get = "pub")]
    memory: Option<u32>,
    #[getset(get = "pub")]
    servers: Option<u32>,
    #[getset(get = "pub")]
    child_node: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub enum LifecycleStatus {
    #[serde(rename = "inactive")]
    #[default]
    Inactive,
    #[serde(rename = "active")]
    Active,
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

impl Capabilities {
    pub fn new(memory: Option<u32>, servers: Option<u32>, child_node: Option<String>) -> Self {
        Self {
            memory,
            servers,
            child_node,
        }
    }
}

impl From<SetActiveError> for Status {
    fn from(val: SetActiveError) -> Self {
        match val {
            SetActiveError::NodeInUseByGroup => Status::unavailable("Node in use by some group"),
            SetActiveError::NodeInUseByServer => Status::unavailable("Node in use by some server"),
            SetActiveError::Error(error) => Status::internal(format!("Error: {error}")),
        }
    }
}
