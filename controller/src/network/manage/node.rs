use anyhow::Result;
use tonic::{Status, async_trait};
use url::Url;

use crate::{
    application::{
        Controller,
        node::{Capabilities, Node},
    },
    network::proto::manage::node::{self, Detail, List, Short},
    task::{BoxedAny, GenericTask, network::TonicTask},
};

pub struct CreateNodeTask(pub String, pub String, pub Capabilities, pub Url);
pub struct UpdateNodeTask(pub String, pub Option<Capabilities>, pub Option<Url>);
pub struct GetNodeTask(pub String);
pub struct GetNodesTask();

#[async_trait]
impl GenericTask for CreateNodeTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        if let Err(error) = controller
            .nodes
            .create_node(&self.0, &self.1, &self.2, &self.3, &controller.plugins)
            .await
        {
            return TonicTask::new_err(error.into());
        }
        TonicTask::new_empty()
    }
}

#[async_trait]
impl GenericTask for UpdateNodeTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        match controller
            .nodes
            .update_node(&self.0, self.1.as_ref(), self.2.as_ref())
            .await
        {
            Ok(node) => return TonicTask::new_ok(Detail::from(node)),
            Err(error) => TonicTask::new_err(error.into()),
        }
    }
}

#[async_trait]
impl GenericTask for GetNodeTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(node) = controller.nodes.get_node(&self.0) else {
            return TonicTask::new_err(Status::not_found("Node not found"));
        };

        TonicTask::new_ok(Detail::from(node))
    }
}

#[async_trait]
impl GenericTask for GetNodesTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        TonicTask::new_ok(List {
            nodes: controller
                .nodes
                .get_nodes()
                .iter()
                .map(std::convert::Into::into)
                .collect(),
        })
    }
}

impl From<&&Node> for Short {
    fn from(node: &&Node) -> Self {
        Self {
            name: node.name().clone(),
        }
    }
}

impl From<&Node> for Detail {
    fn from(value: &Node) -> Self {
        Self {
            name: value.name().clone(),
            plugin: value.plugin().clone(),
            capabilities: Some(value.capabilities().into()),
            controller_address: value.controller().to_string(),
        }
    }
}

impl From<&Capabilities> for node::Capabilities {
    fn from(value: &Capabilities) -> Self {
        Self {
            memory: *value.memory(),
            servers: *value.servers(),
            child_node: value.child_node().clone(),
        }
    }
}
