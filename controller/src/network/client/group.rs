use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::{group::Group, Controller},
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
                .map(|group| group.name().clone())
                .collect(),
        })
    }
}

impl From<&&Group> for Short {
    fn from(group: &&Group) -> Self {
        Self {
            name: group.name().clone(),
        }
    }
}
