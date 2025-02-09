use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    task::{BoxedAny, GenericTask, Task},
};

pub struct TransferUsersTask();

#[async_trait]
impl GenericTask for TransferUsersTask {
    async fn run(&mut self, _controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(0 as u32)
    }
}
