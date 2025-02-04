use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask},
};

pub struct GetServerTask {}
pub struct GetServersTask {}

#[async_trait]
impl GenericTask for GetServerTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}

#[async_trait]
impl GenericTask for GetServersTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}
