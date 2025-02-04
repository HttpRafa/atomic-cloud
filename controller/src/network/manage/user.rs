use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask},
};

pub struct GetUsersTask {}

#[async_trait]
impl GenericTask for GetUsersTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}
