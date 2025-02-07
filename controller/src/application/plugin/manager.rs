use std::collections::HashMap;

use anyhow::Result;
use futures::future::join_all;
use simplelog::info;

use crate::config::Config;

use super::BoxedPlugin;

#[cfg(feature = "wasm-plugins")]
use crate::application::plugin::runtime::wasm::init::init_wasm_plugins;

pub struct PluginManager {
    plugins: HashMap<String, BoxedPlugin>,
}

impl PluginManager {
    pub async fn init(config: &Config) -> Result<Self> {
        info!("Loading plugins...");

        let mut plugins = HashMap::new();

        #[cfg(feature = "wasm-plugins")]
        init_wasm_plugins(config, &mut plugins).await?;

        info!("Loaded {} plugin(s)", plugins.len());
        Ok(Self { plugins })
    }

    pub fn get_plugin(&self, name: &str) -> Option<&BoxedPlugin> {
        self.plugins.get(name)
    }
}

// Ticking
impl PluginManager {
    pub async fn tick(&mut self) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.tick();
        }
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        let tasks = join_all(self.plugins.values().map(|plugin| plugin.shutdown())).await;

        for task in tasks {
            if let Err(error) = task {
                return Err(error.into());
            } else if let Ok(Err(error)) = task {
                return Err(error);
            }
        }
        Ok(())
    }
}
