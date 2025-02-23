use std::collections::HashMap;

use anyhow::Result;
use futures::future::join_all;
use simplelog::info;
use tick::Ticker;

use crate::config::Config;

use super::BoxedPlugin;

#[cfg(feature = "wasm-plugins")]
use crate::application::plugin::runtime::wasm::init::init_wasm_plugins;

mod tick;

pub struct PluginManager {
    plugins: HashMap<String, BoxedPlugin>,

    ticker: Ticker,
}

impl PluginManager {
    pub async fn init(config: &Config) -> Result<Self> {
        info!("Loading plugins...");

        let mut plugins = HashMap::new();

        #[cfg(feature = "wasm-plugins")]
        init_wasm_plugins(config, &mut plugins).await?;

        info!("Loaded {} plugin(s)", plugins.len());
        Ok(Self {
            plugins,
            ticker: Ticker::new(),
        })
    }

    pub fn get_plugins_keys(&self) -> Vec<&String> {
        self.plugins.keys().collect()
    }

    pub fn get_plugin(&self, name: &str) -> Option<&BoxedPlugin> {
        self.plugins.get(name)
    }
}

// Ticking
impl PluginManager {
    #[allow(clippy::unnecessary_wraps)]
    pub async fn tick(&mut self) -> Result<()> {
        self.ticker.tick(&self.plugins).await?;
        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        let tasks = join_all(self.plugins.values().map(|plugin| plugin.shutdown())).await;

        for task in tasks {
            if let Err(error) = task {
                return Err(error.into());
            } else if let Ok(Err(error)) = task {
                return Err(error);
            }
        }

        for (_, mut plugin) in self.plugins.drain() {
            // Before we can drop the plugin we have to drop the wasm resources first
            plugin.cleanup().await?;
            drop(plugin); // Drop the plugin
        }
        Ok(())
    }
}
