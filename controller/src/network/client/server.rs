use anyhow::Result;
use tonic::{async_trait, Status};
use uuid::Uuid;

use crate::{
    application::{server::Server, Controller},
    network::proto::common::common_server::{List, Short},
    task::{BoxedAny, GenericTask, Task},
};

pub struct GetServerTask(pub Uuid);
pub struct GetServerFromNameTask(pub String);
pub struct GetServersTask;

#[async_trait]
impl GenericTask for GetServerTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = controller.servers.get_server(&self.0) else {
            return Task::new_err(Status::not_found("Server not found"));
        };

        Task::new_ok(Short::from(&server))
    }
}

#[async_trait]
impl GenericTask for GetServerFromNameTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = controller.servers.get_server_from_name(&self.0) else {
            return Task::new_err(Status::not_found("Server not found"));
        };

        Task::new_ok(Short::from(&server))
    }
}

#[async_trait]
impl GenericTask for GetServersTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            servers: controller
                .servers
                .get_servers()
                .iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl From<&&Server> for Short {
    fn from(server: &&Server) -> Self {
        Self {
            id: server.id().uuid().to_string(),
            name: server.id().name().clone(),
            group: server.group().clone(),
            node: server.node().clone(),
        }
    }
}
