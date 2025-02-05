use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask, Task},
};

pub struct SetReadyTask {
    pub server: Uuid,
    pub ready: bool,
}

#[async_trait]
impl GenericTask for SetReadyTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match controller.servers_mut().get_server_mut(&self.server) {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        server.set_ready(self.ready);
        Task::new_empty()
    }
}