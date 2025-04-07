use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    network::proto::client::cloudGroup::List,
    task::{BoxedAny, GenericTask, Task},
};

pub struct GetGroupsTask();

#[async_trait]
impl GenericTask for GetGroupsTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            groups: controller
                .groups
                .get_groups()
                .iter()
                .map(|cloudGroup| cloudGroup.name().clone())
                .collect(),
        })
    }
}
