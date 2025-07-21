use std::any::Any;

use anyhow::{Result, anyhow};
use tokio::sync::oneshot::Sender;
use tonic::async_trait;

use crate::application::Controller;

pub mod manager;

pub mod network;
pub mod plugin;

pub type BoxedTask = Box<dyn GenericTask + Send>;
pub type BoxedAny = Box<dyn Any + Send>;

pub struct Task {
    task: BoxedTask,
    sender: Sender<Result<BoxedAny>>,
}

impl Task {
    pub async fn run(mut self, controller: &mut Controller) -> Result<()> {
        let task = self.task.run(controller).await;
        self.sender
            .send(task)
            .map_err(|_| anyhow!("Failed to send task result to the task sender"))
    }
}

#[async_trait]
pub trait GenericTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny>;
}
