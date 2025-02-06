use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask, Task},
};

pub struct UserConnectedTask {
    pub server: Uuid,
    pub uuid: Uuid,
    pub name: String,
}

pub struct UserDisconnectedTask {
    pub server: Uuid,
    pub uuid: Uuid,
}

#[async_trait]
impl GenericTask for UserConnectedTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match controller.servers.get_server_mut(&self.server) {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        controller
            .users
            .user_connected(server, self.name.clone(), self.uuid);
        todo!()
    }
}

#[async_trait]
impl GenericTask for UserDisconnectedTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match controller.servers.get_server_mut(&self.server) {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        controller.users.user_disconnected(server, &self.uuid);
        todo!()
    }
}
