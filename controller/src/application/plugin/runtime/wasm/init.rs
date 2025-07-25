use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::Result;
use common::error::FancyError;
use simplelog::{debug, error, info, warn};
use tokio::{fs, sync::Mutex};
use wasmtime::{
    Cache, Engine, Store,
    component::{Component, HasSelf, Linker},
};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, p2::WasiCtxBuilder};

use crate::{
    application::{
        Shared,
        plugin::{BoxedPlugin, Features, GenericPlugin, runtime::source::Source},
    },
    config::Config,
    storage::Storage,
    task::manager::TaskSender,
};

use super::{
    Plugin, PluginState,
    config::{Permissions, PluginsConfig, verify_engine_config},
    epoch::EpochInvoker,
    generated,
};

#[allow(clippy::too_many_lines)]
pub async fn init_wasm_plugins(
    global_config: &Config,
    tasks: &TaskSender,
    shared: &Arc<Shared>,
    plugins: &mut HashMap<String, BoxedPlugin>,
) -> Result<()> {
    // Verify and load required configuration files
    verify_engine_config().await?;
    let plugins_config = PluginsConfig::parse().await?;

    let directory = Storage::plugins_directory();
    if !directory.exists() {
        fs::create_dir_all(&directory).await?;
    }

    let amount = plugins.len();
    let mut invoker = EpochInvoker::new();
    for (path, _, name) in Storage::for_each_content(&directory).await? {
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm"))
        {
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
                        "Failed to create configs directory for plugin {}: {}",
                        name, error
                    );
                });
        }
        if !data_directory.exists() {
            fs::create_dir_all(&data_directory)
                .await
                .unwrap_or_else(|error| {
                    warn!(
                        "Failed to create data directory for plugin {}: {}",
                        name, error
                    );
                });
        }

        info!("Compiling plugin {}...", name);
        let plugin = Plugin::new(
            &name,
            &source,
            global_config,
            tasks.clone(),
            shared.clone(),
            &plugins_config,
            &data_directory,
            &config_directory,
            &mut invoker,
        )
        .await;
        match plugin {
            Ok(mut plugin) => match plugin.init().await {
                Ok(information) => {
                    if information.ready {
                        info!(
                            "Loaded plugin {} v{} by {}",
                            name,
                            information.version,
                            information.authors.join(", ")
                        );
                        plugin.features = information.features;

                        // Initialize the plugin listener
                        if plugin.features.contains(Features::LISTENER) {
                            match plugin.init_listener().await {
                                Ok(mut listener) => {
                                    listener.register(shared).await;
                                    debug!(
                                        "The plugin {} now listens to the specified events",
                                        name
                                    );
                                    plugin.listener = Some(Arc::new(Mutex::new(listener)));
                                }
                                Err(error) => {
                                    error!(
                                        "Failed to initialize listener for plugin {}: {}",
                                        name, error
                                    );
                                    FancyError::print_fancy(&error, false);
                                }
                            }
                        }

                        plugins.insert(name, Box::new(plugin));
                    } else {
                        warn!("Plugin {} marked itself as not ready, skipping...", name);
                        if let Err(error) = plugin.cleanup().await {
                            error!("Failed to drop resources for plugin {}: {}", name, error);
                            FancyError::print_fancy(&error, false);
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to initialize plugin {}: {}", name, error);
                    FancyError::print_fancy(&error, false);
                }
            },
            Err(error) => {
                error!(
                    "Failed to compile plugin {} at location {}: {}",
                    name, source, error
                );
                FancyError::print_fancy(&error, false);
            }
        }
    }

    if amount == plugins.len() {
        warn!("The Wasm plugins feature is enabled, but no Wasm plugins were loaded.");
    } else {
        invoker.spawn();
    }

    Ok(())
}

impl Plugin {
    #[allow(clippy::too_many_arguments)]
    async fn new(
        name: &str,
        source: &Source,
        global_config: &Config,
        tasks: TaskSender,
        shared: Arc<Shared>,
        plugins_config: &PluginsConfig,
        data_directory: &Path,
        config_directory: &Path,
        invoker: &mut EpochInvoker,
    ) -> Result<Self> {
        let mut engine_config = wasmtime::Config::new();
        engine_config
            .wasm_component_model(true)
            .async_support(true)
            .epoch_interruption(true);
        match Cache::from_file(Some(&Storage::wasm_engine_config_file())) {
            Ok(cache) => {
                engine_config.cache(Some(cache));
            }
            Err(error) => {
                warn!("Failed to enable caching for wasmtime engine: {}", error);
            }
        }

        let engine = Engine::new(&engine_config)?;
        let component = Component::from_binary(&engine, source.get_source())?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
        generated::Plugin::add_to_linker::<_, HasSelf<_>>(
            &mut linker,
            |state: &mut PluginState| state,
        )?;

        let mut wasi = WasiCtxBuilder::new();
        let mut permissions = Permissions::empty();
        if let Some(config) = plugins_config.find_config(name) {
            if config
                .get_permissions()
                .contains(Permissions::INHERIT_STDIO)
            {
                wasi.inherit_stdio();
            }
            if config.get_permissions().contains(Permissions::INHERIT_ARGS) {
                wasi.inherit_args();
            }
            if config.get_permissions().contains(Permissions::INHERIT_ENV) {
                wasi.inherit_env();
            }
            if config
                .get_permissions()
                .contains(Permissions::INHERIT_NETWORK)
            {
                wasi.inherit_network();
            }
            if config
                .get_permissions()
                .contains(Permissions::ALLOW_IP_NAME_LOOKUP)
            {
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
            permissions = config.get_permissions().clone();
        }
        let wasi = wasi
            .preopened_dir(
                config_directory,
                "/configs",
                DirPerms::all(),
                FilePerms::all(),
            )?
            .preopened_dir(data_directory, "/data", DirPerms::all(), FilePerms::all())?
            .build();

        let resources = ResourceTable::new();
        let mut store = Store::new(
            &engine,
            PluginState {
                tasks,
                shared,
                name: name.to_string(),
                permissions,
                wasi,
                resources,
            },
        );
        store.epoch_deadline_async_yield_and_update(2);

        let bindings =
            generated::Plugin::instantiate_async(&mut store, &component, &linker).await?;
        let instance = bindings
            .plugin_system_bridge()
            .plugin()
            .call_constructor(&mut store, global_config.identifier())
            .await?;

        // Start thread that calls the increment epoch function
        invoker.push(&engine);

        Ok(Plugin {
            dropped: false,
            features: Features::empty(),
            listener: None,
            engine,
            bindings: Arc::new(bindings),
            store: Arc::new(Mutex::new(store)),
            instance,
        })
    }
}
