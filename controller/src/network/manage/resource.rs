use anyhow::Result;
use tonic::{async_trait, Status};

use crate::{
    application::Controller, network::proto::manage::resource::Category, task::{BoxedAny, GenericTask, Task}
};

pub struct SetResourceTask(pub Category, pub String, pub bool);

pub struct DeleteResourceTask(pub Category, pub String);

#[async_trait]
impl GenericTask for SetResourceTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        match self.0 {
            Category::Node => {
                let node = controller.nodes.get_node_mut(&self.1).ok_or(Status::not_found("Node not found"))?;
                if let Err(error) = node.set_active(self.2, &controller.servers, &controller.groups) {
                    return Task::new_err(error.into());
                }
                Task::new_empty()
            },
            Category::Group => {
                let group = controller.groups.get_group_mut(&self.1).ok_or(Status::not_found("Group not found"))?;
                Task::new_empty()
            },
            _ => {
                Task::new_err(Status::unimplemented("This category is not supported for this action"))
            },
        }
    }
}

#[async_trait]
impl GenericTask for DeleteResourceTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}
