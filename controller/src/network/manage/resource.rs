use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller, network::proto::manage::resource::Category, task::{BoxedAny, GenericTask}
};

pub struct SetResourceTask(pub Category, pub String, pub bool);

pub struct DeleteResourceTask(pub Category, pub String);

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
