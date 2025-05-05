// Tasks that are used by the tonic network code

use std::{any::type_name, borrow::Cow};

use anyhow::{anyhow, Result};
use common::error::FancyError;
use simplelog::debug;
use tokio::sync::oneshot::channel;
use tonic::{Request, Status};

use crate::{
    application::auth::{permissions::Permissions, AuthType, Authorization},
    task::Task,
};

use super::{manager::TaskSender, BoxedAny, BoxedTask};

pub const INSUFFICIENT_PERMISSIONS_MESSAGE: &str =
    "Insufficient permissions to perform this action";

pub struct TonicTask;

impl TonicTask {
    #[allow(clippy::result_large_err)]
    pub fn get_auth<T>(auth: AuthType, request: &Request<T>) -> Result<Authorization, Status> {
        match request.extensions().get::<Authorization>() {
            Some(data) if data.is_type(auth) => Ok(data.clone()),
            _ => Err(Status::unauthenticated("Not linked")),
        }
    }

    pub async fn execute_authorized<O: Send + 'static, I, F>(
        auth: AuthType,
        flag: Permissions,
        queue: &TaskSender,
        request: Request<I>,
        task: F,
    ) -> Result<O, Status>
    where
        F: FnOnce(Request<I>, Authorization) -> Result<BoxedTask, Status>,
    {
        Self::execute(auth, queue, request, |request, auth| {
            if auth.is_allowed(flag) {
                task(request, auth)
            } else {
                Err(Status::permission_denied(INSUFFICIENT_PERMISSIONS_MESSAGE))
            }
        })
        .await
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
        debug!(
            "Executing tonic task with a return type of: {}",
            type_name::<O>()
        );
        match Self::create::<O>(queue, task(request, data)?).await {
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
            .inner()?
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

    #[allow(clippy::unnecessary_wraps)]
    pub fn new_ok<T: Send + 'static>(value: T) -> Result<BoxedAny> {
        Ok(Box::new(value))
    }

    pub fn new_empty() -> Result<BoxedAny> {
        Self::new_ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn new_err(value: Status) -> Result<BoxedAny> {
        Ok(Box::new(value))
    }

    pub fn new_permission_error<'a, T>(message: T) -> Result<BoxedAny>
    where
        T: Into<Cow<'a, str>>,
    {
        Self::new_err(Status::permission_denied(message.into()))
    }

    pub fn new_link_error() -> Result<BoxedAny> {
        Self::new_err(Status::failed_precondition(
            "Your token is not linked to the required resource for this action",
        ))
    }
}
