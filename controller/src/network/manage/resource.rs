use anyhow::Result;
use tonic::{async_trait, Status};
use uuid::Uuid;

use crate::{
    application::{server::manager::StopRequest, Controller},
    network::proto::manage::resource::Category,
    task::{BoxedAny, GenericTask, Task},
};

pub struct SetResourceTask(pub Category, pub String, pub bool);

pub struct DeleteResourceTask(pub Category, pub String);

#[async_trait]
impl GenericTask for SetResourceTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        match self.0 {
            Category::Node => {
                let node = controller
                    .nodes
                    .get_node_mut(&self.1)
                    .ok_or(Status::not_found("Node not found"))?;
                if let Err(error) = node.set_active(self.2).await {
                    return Task::new_err(Status::internal(error.to_string()));
                }
                Task::new_empty()
            }
            Category::Group => {
                let group = controller
                    .groups
                    .get_group_mut(&self.1)
                    .ok_or(Status::not_found("Group not found"))?;
                if let Err(error) = group.set_active(self.2, &mut controller.servers).await {
                    return Task::new_err(Status::internal(error.to_string()));
                }
                Task::new_empty()
            }
            Category::Server => Task::new_err(Status::unimplemented(
                "This category is not supported for this action",
            )),
        }
    }
}

#[async_trait]
impl GenericTask for DeleteResourceTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        match self.0 {
            Category::Node => {
                if let Err(error) = controller
                    .nodes
                    .delete_node(&self.1, &controller.servers, &controller.groups)
                    .await
                {
                    return Task::new_err(error.into());
                }
                Task::new_empty()
            }
            Category::Group => {
                if let Err(error) = controller.groups.delete_group(&self.1).await {
                    return Task::new_err(error.into());
                }
                Task::new_empty()
            }
            Category::Server => {
                let Ok(uuid) = Uuid::parse_str(&self.1) else {
                    return Task::new_err(Status::invalid_argument("Invalid UUID"));
                };
                let id = match controller.servers.get_server(&uuid) {
                    Some(server) => server.id().clone(),
                    None => return Task::new_err(Status::not_found("Server not found")),
                };

                controller.servers.schedule_stop(StopRequest::new(None, id));
                Task::new_empty()
            }
        }
    }
}
