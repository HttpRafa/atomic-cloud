use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask},
};

pub struct SetReadyTask {
    pub server: Uuid,
    pub ready: bool,
}

#[async_trait]
impl GenericTask for SetReadyTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        Ok(Box::new(()))
    }
}
