use std::{collections::HashMap, fs::{self, DirBuilder}};

use anyhow::{anyhow, Result};
use simplelog::{error, info, warn};

use crate::{application::plugin::{runtime::source::Source, WrappedPlugin}, config::{self, Config}, storage::Storage};

use super::{config::{verify_engine_config, PluginConfig, PluginsConfig}, Plugin};

pub async fn init_wasm_plugins(global_config: &Config, plugins: &mut HashMap<String, WrappedPlugin>) -> Result<()> {
    // Verify and load required configuration files
    verify_engine_config()?;
    let plugins_config = PluginsConfig::parse()?;

    let directory = Storage::get_plugins_directory();
    if !directory.exists() {
        fs::create_dir_all(&directory)?;
    }

    for entry in fs::read_dir(directory)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                error!("Failed to read plugin entry: <red>{}</>", error);
                continue;
            }
        };

        let path = entry.path();
        if path.is_dir()
            || !path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".wasm")
        {
            continue;
        }

        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let source = match Source::from_file(&path) {
            Ok(source) => source,
            Err(error) => {
                error!(
                    "Failed to read source code for plugin {} from file({:?}): {}",
                    name,
                    path,
                    error
                );
                continue;
            }
        };

        info!("Compiling plugin '{}'...", name);
        let plugin = Plugin::new(global_config, &plugins_config).await;
        match plugin {
            Ok(plugin) => match plugin.init().await {
                Ok(info) => {
                    if info.ready {
                        info!(
                            "Loaded plugin {} v{} by {}",
                            plugin.name, info.version,
                            info.authors.join(", ")
                        );
                        plugins.insert(plugin.name, Box::new(plugin));
                    } else {
                        warn!(
                            "Plugin {} marked itself as not ready, skipping...",
                            plugin.name
                        );
                    }
                }
                Err(error) => error!("Failed to load plugin {}: {}", name, error),
            },
            Err(error) => error!(
                "Failed to compile plugin {} at location {}: {}",
                name,
                source,
                error
            ),
        }
    }

    Ok(())
}

impl Plugin {
    async fn new(global_config: &Config, plugins_config: &PluginsConfig) -> Result<Self> {

    }
}