use std::sync::Arc;

use anyhow::{Result, anyhow};
use tokio::spawn;
use tonic::async_trait;

use crate::application::server::screen::{
    GenericScreen, ScreenError, ScreenPullJoinHandle, ScreenWriteJoinHandle,
};

use super::{Context, process::ProcessError};

/// The screen of a server, read straight from the host process registry.
/// Unlike the `wasm` runtime, pulling a screen never enters the plugin.
pub struct Screen {
    context: Arc<Context>,
    process: Option<String>,
}

impl Screen {
    pub fn new(context: Arc<Context>, process: Option<String>) -> Self {
        Self { context, process }
    }
}

#[async_trait]
impl GenericScreen for Screen {
    fn is_supported(&self) -> bool {
        self.process.is_some()
    }

    fn pull(&self) -> ScreenPullJoinHandle {
        let Some(process) = self.process.clone() else {
            return spawn(async { Err(ScreenError::Unsupported) });
        };
        let context = self.context.clone();
        spawn(async move {
            match context.processes.read_lines(&process).await {
                Ok(lines) => Ok(lines),
                // The process is gone (the server stopped or crashed): there
                // is simply nothing more to show.
                Err(ProcessError::NotFound(_)) => Ok(vec![]),
                Err(error) => Err(ScreenError::Error(anyhow!(error))),
            }
        })
    }

    fn write(&self, data: &[u8]) -> ScreenWriteJoinHandle {
        let Some(process) = self.process.clone() else {
            return spawn(async { Err(ScreenError::Unsupported) });
        };
        let context = self.context.clone();
        let data = data.to_vec();
        spawn(async move {
            context
                .processes
                .write(&process, &data)
                .await
                .map_err(|error| ScreenError::Error(anyhow!(error)))
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}
