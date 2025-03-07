use anyhow::Result;
use tonic::{async_trait, Status};
use url::Url;

use crate::{
    application::{
        node::{Capabilities, Node},
        Controller,
    },
    network::proto::manage::node::{Item, List},
    task::{BoxedAny, GenericTask, Task},
};

pub struct CreateNodeTask(pub String, pub String, pub Capabilities, pub Url);
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
            return Task::new_err(error.into());
        }
        Task::new_empty()
    }
}

#[async_trait]
impl GenericTask for GetNodeTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(node) = controller.nodes.get_node(&self.0) else {
            return Task::new_err(Status::not_found("Node not found"));
        };

        Task::new_ok(Item::from(node))
    }
}

#[async_trait]
impl GenericTask for GetNodesTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            nodes: controller
                .nodes
                .get_nodes()
                .iter()
                .map(|node| node.name().clone())
                .collect(),
        })
    }
}

impl From<&Node> for Item {
    fn from(value: &Node) -> Self {
        Self {
            name: value.name().clone(),
            plugin: value.plugin().to_string(),
            memory: *value.capabilities().memory(),
            max: *value.capabilities().max_servers(),
            child: value.capabilities().child().clone(),
            ctrl_addr: value.controller().to_string(),
        }
    }
}
