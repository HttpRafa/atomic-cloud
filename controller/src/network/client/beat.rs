use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask},
};

pub struct BeatTask {
    pub server: Uuid,
}

#[async_trait]
impl GenericTask for BeatTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        Ok(Box::new(()))
    }
}
