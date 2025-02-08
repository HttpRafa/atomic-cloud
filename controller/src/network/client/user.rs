use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::{
        auth::{ActionResult, Authorization},
        server::NameAndUuid,
        Controller,
    },
    task::{BoxedAny, GenericTask, Task},
};

pub struct UserConnectedTask(pub Authorization, pub NameAndUuid);
pub struct UserDisconnectedTask(pub Authorization, pub Uuid);

#[async_trait]
impl GenericTask for UserConnectedTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match self
            .0
            .get_server()
            .and_then(|server| controller.servers.get_server_mut(server.uuid()))
        {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        controller.users.user_connected(server, self.1.clone());
        Task::new_empty()
    }
}

#[async_trait]
impl GenericTask for UserDisconnectedTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match self
            .0
            .get_server()
            .and_then(|server| controller.servers.get_server_mut(server.uuid()))
        {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        if controller.users.user_disconnected(server, &self.1) == ActionResult::Denied {
            return Task::new_permission_error("You are not allowed to disconnect this user");
        }
        Task::new_empty()
    }
}
