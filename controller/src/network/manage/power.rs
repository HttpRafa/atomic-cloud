use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask, network::TonicTask},
};

pub struct RequestStopTask();

#[async_trait]
impl GenericTask for RequestStopTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        controller.signal_shutdown();
        TonicTask::new_empty()
    }
}
