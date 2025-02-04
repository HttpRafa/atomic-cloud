use std::any::Any;

use anyhow::{anyhow, Result};
use common::error::CloudError;
use tokio::sync::oneshot::{channel, Sender};
use tonic::{async_trait, Request, Status};

use crate::application::{Controller, TaskSender};

pub type BoxedTask = Box<dyn GenericTask + Send>;
pub type BoxedAny = Box<dyn Any + Send>;

pub struct Task {
    task: BoxedTask,
    sender: Sender<Result<BoxedAny>>,
}

impl Task {
    pub async fn execute_task<O: Send + 'static, D: Clone + Send + Sync + 'static, I, F>(
        queue: &TaskSender,
        request: &mut Request<I>,
        task: F,
    ) -> Result<O, Status>
    where
        F: FnOnce(&mut Request<I>, D) -> BoxedTask,
    {
        let data = match request.extensions().get::<D>() {
            Some(data) => data,
            None => return Err(Status::permission_denied("Not linked")),
        }
        .clone();
        match Task::create::<O>(queue, task(request, data)).await {
            Ok(value) => Ok(value),
            Err(error) => {
                CloudError::print_fancy(&error, false);
                Err(Status::internal(error.to_string()))
            }
        }
    }

    pub async fn create<T: Send + 'static>(queue: &TaskSender, task: BoxedTask) -> Result<T> {
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
