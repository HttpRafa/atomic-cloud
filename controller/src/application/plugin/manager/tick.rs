use std::collections::HashMap;

use anyhow::Result;
use common::error::FancyError;
use futures::future::join_all;
use simplelog::warn;
use tokio::task::JoinHandle;

use crate::application::plugin::BoxedPlugin;

pub struct Ticker(HashMap<String, JoinHandle<Result<()>>>);

impl Ticker {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub async fn tick(&mut self, plugins: &HashMap<String, BoxedPlugin>) -> Result<()> {
        for result in join_all(
            self.0
                .extract_if(|_, handle| handle.is_finished())
                .map(|(_, handle)| handle),
        )
        .await
        {
            match result.map_err(Into::into) {
                Ok(Ok(())) => {}
                Ok(Err(error)) | Err(error) => {
                    warn!("Plugin failed to tick: {:?}", error);
                    FancyError::print_fancy(&error, false);
                }
            }
        }

        for (name, plugin) in plugins {
            self.0.entry(name.clone()).or_insert_with(|| plugin.tick());
        }
        Ok(())
    }
}
