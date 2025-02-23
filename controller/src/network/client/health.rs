use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::{
        auth::Authorization,
        server::{manager::StopRequest, State},
        Controller,
    },
    task::{BoxedAny, GenericTask, Task},
};

pub struct SetRunningTask(pub Authorization);
pub struct RequestStopTask(pub Authorization);

#[async_trait]
impl GenericTask for SetRunningTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = self
            .0
            .get_server()
            .and_then(|server| controller.servers.get_server_mut(server.uuid()))
        else {
            return Task::new_link_error();
        };
        server.set_state(State::Running);
        Task::new_empty()
    }
}

#[async_trait]
impl GenericTask for RequestStopTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = self
            .0
            .get_server()
            .and_then(|server| controller.servers.resolve_server(server.uuid()))
        else {
            return Task::new_link_error();
        };
        controller
            .servers
            .schedule_stop(StopRequest::new(None, server));
        Task::new_empty()
    }
}
