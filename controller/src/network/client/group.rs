use anyhow::Result;
use tonic::{async_trait, Status};

use crate::{
    application::{group::Group, Controller},
    network::proto::common::common_group::{List, Short},
    task::{network::TonicTask, BoxedAny, GenericTask},
};

pub struct GetGroupTask(pub String);
pub struct GetGroupsTask;

#[async_trait]
impl GenericTask for GetGroupTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(group) = controller.groups.get_group(&self.0) else {
            return TonicTask::new_err(Status::not_found("Group not found"));
        };

        TonicTask::new_ok(Short::from(&group))
    }
}

#[async_trait]
impl GenericTask for GetGroupsTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        TonicTask::new_ok(List {
            groups: controller
                .groups
                .get_groups()
                .iter()
                .map(Into::into)
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
