use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask},
};

pub struct UserConnectedTask {
    server: Uuid,
}

pub struct UserDisconnectedTask {
    server: Uuid,
}

#[async_trait]
impl GenericTask for UserConnectedTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}

#[async_trait]
impl GenericTask for UserDisconnectedTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}
