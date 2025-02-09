use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::{server::Server, Controller},
    network::proto::manage::server::{List, Short},
    task::{BoxedAny, GenericTask, Task},
};

pub struct GetServerTask();
pub struct GetServersTask();

#[async_trait]
impl GenericTask for GetServerTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
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
                .map(std::convert::Into::into)
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
