use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::Result;
use common::file::for_each_content;
use simplelog::{error, info, warn};
use tokio::{fs, sync::Mutex};
use wasmtime::{
    component::{Component, Linker},
    Engine, Store,
};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtxBuilder};

use crate::{
    application::plugin::{runtime::source::Source, GenericPlugin, WrappedPlugin},
    config::Config,
    storage::Storage,
};

use super::{
    config::{verify_engine_config, PluginsConfig},
    generated, Plugin, PluginState,
};

pub async fn init_wasm_plugins(
    global_config: &Config,
    plugins: &mut HashMap<String, WrappedPlugin>,
) -> Result<()> {
    // Verify and load required configuration files
    verify_engine_config()?;
    let plugins_config = PluginsConfig::parse()?;

    let directory = Storage::plugins_directory();
    if !directory.exists() {
        fs::create_dir_all(&directory).await?;
    }

    let amount = plugins.len();
    for (path, file_name, name) in for_each_content(&directory)? {
        if !file_name.ends_with(".wasm") {
            continue;
        }

        let source = match Source::from_file(&path) {
            Ok(source) => source,
            Err(error) => {
                error!(
                    "Failed to read source code for plugin {} from file({:?}): {}",
                    name, path, error
                );
                continue;
            }
        };

        let config_directory = Storage::config_directory_for_plugin(&name);
        let data_directory = Storage::data_directory_for_plugin(&name);
        if !config_directory.exists() {
            fs::create_dir_all(&config_directory)
                .await
                .unwrap_or_else(|error| {
                    warn!(
                        "Failed to create configs directory for driver {}: {}",
                        name, error
                    )
                });
        }
        if !data_directory.exists() {
            fs::create_dir_all(&data_directory)
                .await
                .unwrap_or_else(|error| {
                    warn!(
                        "Failed to create data directory for driver {}: {}",
                        name, error
                    )
                });
        }

        info!("Compiling plugin '{}'...", name);
        let plugin = Plugin::new(
            &name,
            &source,
            global_config,
            &plugins_config,
            &data_directory,
            &config_directory,
        )
        .await;
        match plugin {
            Ok(plugin) => match plugin.init().await {
                Ok(information) => {
                    if information.ready {
                        info!(
                            "Loaded plugin {} v{} by {}",
                            name,
                            information.version,
                            information.authors.join(", ")
                        );
                        plugins.insert(name, Box::new(plugin));
                    } else {
                        warn!("Plugin {} marked itself as not ready, skipping...", name);
                    }
                }
                Err(error) => error!("Failed to initialize plugin {}: {}", name, error),
            },
            Err(error) => error!(
                "Failed to compile plugin {} at location {}: {}",
                name, source, error
            ),
        }
    }

    if amount == plugins.len() {
        warn!("The Wasm plugins feature is enabled, but no Wasm plugins were loaded.");
    }

    Ok(())
}

impl Plugin {
    async fn new(
        name: &str,
        source: &Source,
        global_config: &Config,
        plugins_config: &PluginsConfig,
        data_directory: &Path,
        config_directory: &Path,
    ) -> Result<Self> {
        let mut engine_config = wasmtime::Config::new();
        engine_config
            .wasm_component_model(true)
            .async_support(true)
            .epoch_interruption(true);
        if let Err(error) = engine_config.cache_config_load(Storage::wasm_engine_config_file()) {
            warn!("Failed to enable caching for wasmtime engine: {}", error);
        }

        let engine = Engine::new(&engine_config)?;
        let component = Component::from_binary(&engine, source.get_source())?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        generated::Plugin::add_to_linker(&mut linker, |state: &mut PluginState| state)?;

        let mut wasi = WasiCtxBuilder::new();
        if let Some(config) = plugins_config.find_config(name) {
            if config.has_inherit_stdio() {
                wasi.inherit_stdio();
            }
            if config.has_inherit_args() {
                wasi.inherit_args();
            }
            if config.has_inherit_env() {
                wasi.inherit_env();
            }
            if config.has_inherit_network() {
                wasi.inherit_network();
            }
            if config.has_allow_ip_name_lookup() {
                wasi.allow_ip_name_lookup(true);
            }
            for mount in config.get_mounts() {
                wasi.preopened_dir(
                    mount.get_host(),
                    mount.get_guest(),
                    DirPerms::all(),
                    FilePerms::all(),
                )?;
            }
        }
        let wasi = wasi
            .preopened_dir(
                config_directory,
                "/configs/",
                DirPerms::all(),
                FilePerms::all(),
            )?
            .preopened_dir(data_directory, "/data/", DirPerms::all(), FilePerms::all())?
            .build();

        let resources = ResourceTable::new();
        let mut store = Store::new(
            &engine,
            PluginState {
                name: name.to_string(),
                wasi,
                resources,
            },
        );

        let bindings =
            generated::Plugin::instantiate_async(&mut store, &component, &linker).await?;
        let instance = bindings
            .plugin_system_bridge()
            .generic_plugin()
            .call_constructor(&mut store, global_config.identifier())
            .await?;

        Ok(Plugin {
            bindings: Arc::new(bindings),
            store: Arc::new(Mutex::new(store)),
            instance,
        })
    }
}
