use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::Controller,
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
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_empty()
    }
}

#[async_trait]
impl GenericTask for RequestStopTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_empty()
    }
}
