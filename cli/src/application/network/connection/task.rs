use std::{future::Future, time::Duration};

use color_eyre::eyre::{eyre, Result};
use tokio::{task::JoinHandle, time::Instant};

use super::EstablishedConnection;

pub type ConnectTask = NetworkTask<Result<EstablishedConnection>>;
pub type EmptyTask = NetworkTask<Result<()>>;
//pub type DataTask<T> = NetworkTask<Result<T>>;

pub struct NetworkTask<T> {
    instant: Instant,
    handle: Option<JoinHandle<T>>,
}

impl<T> NetworkTask<T> {
    pub fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }

    pub fn abort(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    pub async fn get(&mut self) -> Result<T> {
        if self.handle.is_some() {
            Ok(self
                .handle
                .take()
                .expect("Can not be None because of the if above")
                .await?)
        } else {
            Err(eyre!("Task has already been pulled"))
        }
    }

    pub async fn get_now(&mut self) -> Result<Option<T>> {
        match &mut self.handle {
            Some(handle) if handle.is_finished() => Ok(Some(
                self.handle
                    .take()
                    .expect("Can not be None because of the if above")
                    .await?,
            )),
            _ => Ok(None),
        }
    }
}

pub fn spawn<F>(future: F) -> NetworkTask<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    NetworkTask {
        instant: Instant::now(),
        handle: Some(tokio::spawn(future)),
    }
}
