use anyhow::Result;
use simplelog::debug;
use tonic::{async_trait, Status};
use uuid::Uuid;

use crate::{
    application::{
        node::Allocation,
        server::{manager::StartRequest, Resources, Server, Specification},
        Controller,
    },
    network::proto::{
        common::{common_server::List, Address},
        manage::server::{self, Detail},
    },
    task::{BoxedAny, GenericTask, Task},
};

pub struct ScheduleServerTask(
    pub i32,
    pub String,
    pub String,
    pub Resources,
    pub Specification,
);
pub struct GetServerTask(pub Uuid);
pub struct GetServerFromNameTask(pub String);
pub struct GetServersTask;

#[async_trait]
impl GenericTask for ScheduleServerTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let request = StartRequest::new(
            None,
            self.0,
            self.1.clone(),
            None,
            &[self.2.clone()],
            &self.3,
            &self.4,
        );
        let uuid = request.id().uuid().to_string();
        debug!(
            "Scheduled server({}) without a group assignment",
            request.id()
        );
        controller.servers.schedule_start(request);

        Task::new_ok(uuid)
    }
}

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
impl GenericTask for GetServerFromNameTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = controller.servers.get_server_from_name(&self.0) else {
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
            specification: Some(value.specification().into()),
        }
    }
}
