// Tasks that are used by the plugin system

use std::any::type_name;

use anyhow::{Result, anyhow};
use simplelog::debug;
use tokio::sync::oneshot::channel;

use crate::task::Task;

use super::{BoxedAny, BoxedTask, GenericTask, manager::TaskSender};

pub struct PluginTask;

impl PluginTask {
    pub async fn execute<O: Send + 'static, T>(queue: &TaskSender, task: T) -> Result<O>
    where
        T: GenericTask + Send + 'static,
    {
        debug!(
            "Executing plugin task with a return type of: {}",
            type_name::<O>()
        );
        match Self::create::<O>(queue, Box::new(task)).await {
            Ok(value) => value,
            Err(error) => Err(error),
        }
    }

    pub async fn create<T: Send + 'static>(
        queue: &TaskSender,
        task: BoxedTask,
    ) -> Result<Result<T>> {
        let (sender, receiver) = channel();
        queue
            .inner()?
            .send(Task { task, sender })
            .await
            .map_err(|_| anyhow!("Failed to send task to task queue"))?;
        let result = receiver.await??;
        match result.downcast::<T>() {
            Ok(result) => Ok(Ok(*result)),
            Err(result) => match result.downcast::<anyhow::Error>() {
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

    pub fn _new_empty() -> Result<BoxedAny> {
        Self::new_ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn _new_err(value: anyhow::Error) -> Result<BoxedAny> {
        Ok(Box::new(value))
    }
}
