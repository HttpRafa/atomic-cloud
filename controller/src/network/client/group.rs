use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::{auth::Authorization, Controller},
    task::{BoxedAny, GenericTask},
};

pub struct GetGroupsTask {
    pub auth: Authorization,
}

#[async_trait]
impl GenericTask for GetGroupsTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        todo!()
    }
}
