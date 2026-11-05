use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::Result;
use common::error::FancyError;
use simplelog::{error, info, warn};
use tokio::fs;
use wasmtime::{
    Cache, Engine,
    component::{Component, HasSelf, Linker},
};

use crate::{
    application::{
        Shared,
        plugin::{
            BoxedPlugin, GenericPlugin,
            runtime::{
                config::{Permissions, PluginsConfig, verify_engine_config},
                epoch::EpochInvoker,
                source::Source,
            },
        },
    },
    config::Config,
    storage::Storage,
};

use super::{
    Context, Runtime, State, generated, kv::KvStore, plugin::Plugin, process::ProcessRegistry,
};

/// Wasmite plugins live in their own subdirectory of the plugins directory,
/// so the `wasm` runtime and the `wasmite` runtime never try to load each
/// other's components.
pub const WASMITE_DIRECTORY: &str = "wasmite";
/// Host-owned files inside a plugin's data directory.
const HOST_DIRECTORY: &str = ".wasmite";

pub struct LoadOptions {
    pub name: String,
    pub identifier: String,
    pub certificate: Option<String>,
    pub permissions: Permissions,
    pub mounts: Vec<(String, String)>,
    pub data_directory: PathBuf,
    pub config_directory: PathBuf,
    pub cache: bool,
}

pub async fn init_wasmite_plugins(
    global_config: &Config,
    shared: &Arc<Shared>,
    plugins: &mut HashMap<String, BoxedPlugin>,
) -> Result<()> {
    verify_engine_config().await?;
    let plugins_config = PluginsConfig::parse().await?;

    let directory = Storage::plugins_directory().join(WASMITE_DIRECTORY);
    if !directory.exists() {
        fs::create_dir_all(&directory).await?;
    }

    let certificate = shared
        .tls
        .tls
        .as_ref()
        .map(|(certificate, _)| certificate.clone());

    let mut loaded = 0;
    let mut invoker = EpochInvoker::new();
    for (path, _, name) in Storage::for_each_content(&directory).await? {
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("wasm"))
        {
            continue;
        }
        if plugins.contains_key(&name) {
            warn!(
                "Skipping wasmite plugin {} because a plugin with the same name is already loaded",
                name
            );
            continue;
        }

        let source = match Source::from_file(&path) {
            Ok(source) => source,
            Err(error) => {
                error!(
                    "Failed to read plugin {} from file({:?}): {}",
                    name, path, error
                );
                continue;
            }
        };

        let config = plugins_config.find_config(&name);
        let options = LoadOptions {
            name: name.clone(),
            identifier: global_config.identifier().to_string(),
            certificate: certificate.clone(),
            permissions: config.map_or_else(Permissions::empty, |config| {
                config.get_permissions().clone()
            }),
            mounts: config.map_or_else(Vec::new, |config| {
                config
                    .get_mounts()
                    .iter()
                    .map(|mount| (mount.get_host().to_string(), mount.get_guest().to_string()))
                    .collect()
            }),
            data_directory: Storage::data_directory_for_plugin(&name),
            config_directory: Storage::config_directory_for_plugin(&name),
            cache: true,
        };

        info!("Compiling wasmite plugin {}...", name);
        match load(options, source.get_source(), &mut invoker).await {
            Ok(plugin) => {
                plugins.insert(name, Box::new(plugin));
                loaded += 1;
            }
            Err(error) => {
                error!("Failed to load wasmite plugin {} from {}", name, source);
                FancyError::print_fancy(&error, false);
            }
        }
    }

    if loaded > 0 {
        invoker.spawn();
    }
    Ok(())
}

/// Compiles, links and initializes one plugin.
pub async fn load(
    options: LoadOptions,
    source: &[u8],
    invoker: &mut EpochInvoker,
) -> Result<Plugin> {
    for directory in [&options.data_directory, &options.config_directory] {
        fs::create_dir_all(directory).await?;
    }

    let mut engine_config = wasmtime::Config::new();
    engine_config
        .wasm_component_model(true)
        .epoch_interruption(true);
    if options.cache {
        match Cache::from_file(Some(&Storage::wasm_engine_config_file())) {
            Ok(cache) => {
                engine_config.cache(Some(cache));
            }
            Err(error) => {
                warn!("Failed to enable caching for wasmtime engine: {}", error);
            }
        }
    }
    let engine = Engine::new(&engine_config)?;
    let component = Component::from_binary(&engine, source)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    generated::Plugin::add_to_linker::<State, HasSelf<State>>(&mut linker, |state| state)?;
    // Linking happens once here; every call only instantiates.
    let pre = generated::PluginPre::new(linker.instantiate_pre(&component)?)?;

    let host_directory = options.data_directory.join(HOST_DIRECTORY);
    let context = Arc::new(Context {
        kv: KvStore::open(host_directory.join("kv.toml")).await?,
        processes: ProcessRegistry::open(
            options.data_directory.clone(),
            host_directory.join("processes.toml"),
        )
        .await?,
        name: options.name.clone(),
        identifier: options.identifier,
        permissions: options.permissions,
        certificate: options.certificate,
        data_directory: options.data_directory,
        config_directory: options.config_directory,
        mounts: options.mounts,
        guards: std::sync::Mutex::new(HashMap::new()),
    });

    invoker.push(&engine);
    let mut plugin = Plugin::new(Runtime::new(engine, pre, context));
    let information = plugin.init().await?;
    info!(
        "Loaded wasmite plugin {} v{} by {}",
        options.name,
        information.version,
        information.authors.join(", ")
    );
    plugin.set_features(information.features);
    Ok(plugin)
}
