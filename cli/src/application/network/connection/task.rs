use std::{future::Future, time::Duration};

use color_eyre::eyre::Result;
use tokio::{task::JoinHandle, time::Instant};

use super::EstablishedConnection;

pub type ConnectTask = NetworkTask<Result<EstablishedConnection>>;

pub struct NetworkTask<T> {
    instant: Instant,
    handle: Option<JoinHandle<T>>,
}

impl<T> NetworkTask<T> {
    pub fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }

    pub async fn get(&mut self) -> Result<Option<T>> {
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
