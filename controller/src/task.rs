use std::any::Any;

use anyhow::{anyhow, Result};
use tokio::sync::oneshot::{channel, Sender};
use tonic::async_trait;

use crate::application::{Controller, TaskSender};

type BoxedTask = Box<dyn GenericTask + Send>;
pub type BoxedAny = Box<dyn Any + Send>;

pub struct Task {
    task: BoxedTask,
    sender: Sender<Result<BoxedAny>>,
}

impl Task {
    pub async fn create<T: Send + 'static>(queue: TaskSender, task: BoxedTask) -> Result<T> {
        let (sender, receiver) = channel();
        queue
            .send(Task { task, sender })
            .await
            .map_err(|_| anyhow!("Failed to send task to task queue"))?;
        Ok(*receiver.await??.downcast::<T>().map_err(|_| {
            anyhow!(
                "Failed to downcast task result to the expected type. Check task implementation"
            )
        })?)
    }

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
