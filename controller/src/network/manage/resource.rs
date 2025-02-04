use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask},
};

pub struct SetResourceTask {}

pub struct DeleteResourceTask {}

#[async_trait]
impl GenericTask for SetResourceTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}

#[async_trait]
impl GenericTask for DeleteResourceTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}
