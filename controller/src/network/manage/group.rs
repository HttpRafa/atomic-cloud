use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller, network::proto::manage::group::List, task::{BoxedAny, GenericTask, Task}
};

pub struct CreateGroupTask();
pub struct GetGroupTask();
pub struct GetGroupsTask();

#[async_trait]
impl GenericTask for CreateGroupTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}

#[async_trait]
impl GenericTask for GetGroupTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}

#[async_trait]
impl GenericTask for GetGroupsTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            groups: controller
                .groups
                .get_groups()
                .iter()
                .map(|group| group.name().clone())
                .collect(),
        })
    }
}
