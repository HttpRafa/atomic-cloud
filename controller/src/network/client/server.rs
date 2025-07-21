use anyhow::Result;
use tonic::{Status, async_trait};
use uuid::Uuid;

use crate::{
    application::{Controller, server::Server},
    network::proto::common::common_server::{List, Short},
    task::{BoxedAny, GenericTask, network::TonicTask},
};

pub struct GetServerTask(pub Uuid);
pub struct GetServerFromNameTask(pub String);
pub struct GetServersTask;

#[async_trait]
impl GenericTask for GetServerTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = controller.servers.get_server(&self.0) else {
            return TonicTask::new_err(Status::not_found("Server not found"));
        };

        TonicTask::new_ok(Short::from(&server))
    }
}

#[async_trait]
impl GenericTask for GetServerFromNameTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = controller.servers.get_server_from_name(&self.0) else {
            return TonicTask::new_err(Status::not_found("Server not found"));
        };

        TonicTask::new_ok(Short::from(&server))
    }
}

#[async_trait]
impl GenericTask for GetServersTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        TonicTask::new_ok(List {
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
