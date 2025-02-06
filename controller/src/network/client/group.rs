use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask},
};

pub struct GetGroupsTask {
    pub server: Uuid,
}

#[async_trait]
impl GenericTask for GetGroupsTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}
