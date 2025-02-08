use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller, network::proto::manage::node::List, task::{BoxedAny, GenericTask, Task}
};

pub struct CreateNodeTask();
pub struct GetNodeTask();
pub struct GetNodesTask();

#[async_trait]
impl GenericTask for CreateNodeTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}

#[async_trait]
impl GenericTask for GetNodeTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
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
