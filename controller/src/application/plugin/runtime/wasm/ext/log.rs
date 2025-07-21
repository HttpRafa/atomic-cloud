use anyhow::Result;
use simplelog::{debug, error, info, warn};

use crate::application::plugin::runtime::wasm::{
    PluginState,
    generated::plugin::system::{self, log::Level},
};

impl system::log::Host for PluginState {
    async fn log_string(&mut self, level: Level, message: String) -> Result<()> {
        match level {
            Level::Info => info!("[{}] {}", self.name.to_uppercase(), message),
            Level::Warn => warn!("[{}] {}", self.name.to_uppercase(), message),
            Level::Error => error!("[{}] {}", self.name.to_uppercase(), message),
            Level::Debug => debug!("[{}] {}", self.name.to_uppercase(), message),
        }

        Ok(())
    }
}
