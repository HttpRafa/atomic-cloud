use std::any::{type_name, Any};

use anyhow::{anyhow, Result};
use common::error::FancyError;
use simplelog::debug;
use tokio::sync::oneshot::{channel, Sender};
use tonic::{async_trait, Request, Status};

use crate::application::{
    auth::{AuthType, Authorization},
    Controller, TaskSender,
};

pub type BoxedTask = Box<dyn GenericTask + Send>;
pub type BoxedAny = Box<dyn Any + Send>;

pub struct Task {
    task: BoxedTask,
    sender: Sender<Result<BoxedAny>>,
}

impl Task {
    pub fn get_auth<T>(auth: AuthType, request: &Request<T>) -> Result<Authorization, Status> {
        match request.extensions().get::<Authorization>() {
            Some(data) if data.is_type(auth) => Ok(data.clone()),
            _ => Err(Status::permission_denied("Not linked")),
        }
    }

    pub async fn execute<O: Send + 'static, I, F>(
        auth: AuthType,
        queue: &TaskSender,
        request: Request<I>,
        task: F,
    ) -> Result<O, Status>
    where
        F: FnOnce(Request<I>, Authorization) -> Result<BoxedTask, Status>,
    {
        let data = Self::get_auth(auth, &request)?;
        debug!("Executing task with a return type of: {}", type_name::<O>());
        match Task::create::<O>(queue, task(request, data)?).await {
            Ok(value) => value,
            Err(error) => {
                FancyError::print_fancy(&error, false);
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

    pub fn new_permission_error(message: &str) -> Result<BoxedAny> {
        Self::new_err(Status::permission_denied(message))
    }

    pub fn new_link_error() -> Result<BoxedAny> {
        Self::new_err(Status::failed_precondition(
            "Your token is not linked to the required resource for this action",
        ))
    }
}

#[async_trait]
pub trait GenericTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny>;
}
