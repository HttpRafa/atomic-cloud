use anyhow::Result;
use tonic::{async_trait, Status};
use uuid::Uuid;

use crate::{
    application::{node::Allocation, server::Server, Controller},
    network::proto::{
        common::Address,
        manage::server::{self, Detail, List, Short},
    },
    task::{BoxedAny, GenericTask, Task},
};

pub struct GetServerTask(pub Uuid);
pub struct GetServersTask();

#[async_trait]
impl GenericTask for GetServerTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = controller.servers.get_server(&self.0) else {
            return Task::new_err(Status::not_found("Server not found"));
        };

        Task::new_ok(Detail::from(server))
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

impl From<&Server> for Detail {
    fn from(server: &Server) -> Self {
        Self {
            name: server.id().name().clone(),
            id: server.id().uuid().to_string(),
            group: server.group().clone(),
            node: server.node().clone(),
            allocation: Some(server.allocation().into()),
            users: *server.connected_users(),
            token: server.token().clone(),
            state: server.state().clone() as i32,
            ready: *server.ready(),
        }
    }
}

impl From<&Allocation> for server::Allocation {
    fn from(value: &Allocation) -> Self {
        Self {
            ports: value
                .ports()
                .iter()
                .map(|port| Address {
                    host: port.host.clone(),
                    port: u32::from(port.port),
                })
                .collect(),
            resources: Some(value.resources().into()),
            spec: Some(value.spec().into()),
        }
    }
}
