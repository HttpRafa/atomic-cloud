use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::{
        server::{manager::StopRequest, State},
        Controller,
    },
    task::{BoxedAny, GenericTask, Task},
};

pub struct SetRunningTask {
    pub server: Uuid,
}

pub struct RequestStopTask {
    pub server: Uuid,
}

#[async_trait]
impl GenericTask for SetRunningTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match controller.servers.get_server_mut(&self.server) {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        server.set_state(State::Running);
        Task::new_empty()
    }
}

#[async_trait]
impl GenericTask for RequestStopTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match controller.servers.resolve_server(&self.server) {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        controller
            .servers
            .schedule_stop(StopRequest::new(None, server));
        Task::new_empty()
    }
}
