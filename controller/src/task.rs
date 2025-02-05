use std::any::{type_name, Any};

use anyhow::{anyhow, Result};
use common::error::CloudError;
use simplelog::debug;
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
    pub async fn execute<O: Send + 'static, D: Clone + Send + Sync + 'static, I, F>(
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
        debug!("Executing task with a return type of: {}", type_name::<O>());
        match Task::create::<O>(queue, task(request, data)).await {
            Ok(value) => value,
            Err(error) => {
                CloudError::print_fancy(&error, false);
                Err(Status::internal(error.to_string()))
            }
        }
    }

    pub async fn create<T: Send + 'static>(
        queue: &TaskSender,
        task: BoxedTask,
    ) -> Result<Result<T, Status>> {
        let (sender, receiver) = channel();
        queue
            .send(Task { task, sender })
            .await
            .map_err(|_| anyhow!("Failed to send task to task queue"))?;
        let result = receiver.await??;
        match result.downcast::<T>() {
            Ok(result) => Ok(Ok(*result)),
            Err(result) => match result.downcast::<Status>() {
                Ok(result) => Ok(Err(*result)),
                Err(_) => Err(anyhow!(
                    "Failed to downcast task result to the expected type. Check task implementation"
                )),
            },
        }
    }

    pub async fn run(mut self, controller: &mut Controller) -> Result<()> {
        let task = self.task.run(controller).await;
        self.sender
            .send(task)
            .map_err(|_| anyhow!("Failed to send task result to the task sender"))
    }

    pub fn new_ok<T: Send + 'static>(value: T) -> Result<BoxedAny> {
        Ok(Box::new(value))
    }
    
    pub fn new_empty() -> Result<BoxedAny> {
        Self::new_ok(())
    }
    
    pub fn new_err(value: Status) -> Result<BoxedAny> {
        Ok(Box::new(value))
    }
    
    pub fn new_link_error() -> Result<BoxedAny> {
        Self::new_err(Status::failed_precondition("Not linked"))
    }
}

#[async_trait]
pub trait GenericTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny>;
}