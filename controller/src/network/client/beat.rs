use anyhow::Result;
use tonic::{async_trait, Status};
use uuid::Uuid;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask, Task},
};

pub struct BeatTask {
    pub server: Uuid,
}

#[async_trait]
impl GenericTask for BeatTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match controller.servers.get_server_mut(&self.server) {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        server.heart_mut().beat();
        Task::new_empty()
    }
}
