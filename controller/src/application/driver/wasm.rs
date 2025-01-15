use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex, RwLock, Weak};

use anyhow::{anyhow, Result};
use cloudlet::WasmCloudlet;
use common::config::LoadFromTomlFile;
use config::WasmConfig;
use generated::exports::cloudlet::driver::bridge;
use generated::Driver;
use simplelog::{error, info, warn};
use wasmtime::component::{Component, Linker, ResourceAny};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use super::source::Source;
use super::{DriverCloudletHandle, GenericDriver, Information};
use crate::application::cloudlet::{Capabilities, Cloudlet, HostAndPort, RemoteController};
use crate::storage::Storage;

mod config;

mod cloudlet;
mod http;
mod log;
mod process;

pub mod generated {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "driver",
        path: "../protocol/wit/",
    });
}

const WASM_DIRECTORY: &str = "wasm";

/* Caching of compiled wasm artifacts and other configuration */
const CONFIG_FILE: &str = "wasm.toml";
const DEFAULT_CONFIG: &str = r#"# For more settings, please refer to the documentation:
# https://bytecodealliance.github.io/wasmtime/cli-cache.html

[cache]
enabled = true

# This section is crucial for granting the drivers their required permissions
# https://httprafa.github.io/atomic-cloud/controller/drivers/wasm/permissions/

[[drivers]]
name = "pterodactyl"
inherit_stdio = false
inherit_args = false
inherit_env = false
inherit_network = true
allow_ip_name_lookup = true
allow_http = true
allow_process = false
mounts = []"#;

struct WasmDriverState {
    handle: Weak<WasmDriver>,
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for WasmDriverState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl generated::cloudlet::driver::api::Host for WasmDriverState {
    fn get_name(&mut self) -> String {
        self.handle.upgrade().unwrap().name.clone()
    }
}

struct WasmDriverHandle {
    store: Store<WasmDriverState>,
    resource: ResourceAny, // This is delete when the store is dropped
}

impl WasmDriverHandle {
    fn new(store: Store<WasmDriverState>, resource: ResourceAny) -> Self {
        WasmDriverHandle { store, resource }
    }

    fn get(&mut self) -> (ResourceAny, &mut Store<WasmDriverState>) {
        (self.resource, &mut self.store)
    }
}

pub struct WasmDriverData {
    child_processes: RwLock<HashMap<u32, std::process::Child>>,
}

pub struct WasmDriver {
    own: Weak<WasmDriver>,

    name: String,
    bindings: Driver,
    handle: Mutex<Option<WasmDriverHandle>>,

    data: WasmDriverData,
}

impl WasmDriver {
    fn get_resource_and_store(
        handle: &mut Option<WasmDriverHandle>,
    ) -> (ResourceAny, &mut Store<WasmDriverState>) {
        handle.as_mut().unwrap().get()
    }
}

impl GenericDriver for WasmDriver {
    fn name(&self) -> &String {
        &self.name
    }

    fn init(&self) -> Result<Information> {
        let mut handle = self.handle.lock().unwrap();
        let (resource, store) = Self::get_resource_and_store(&mut handle);
        match self
            .bindings
            .cloudlet_driver_bridge()
            .generic_driver()
            .call_init(store, resource)
        {
            Ok(information) => Ok(information.into()),
            Err(error) => Err(error),
        }
    }

    fn init_cloudlet(&self, cloudlet: &Cloudlet) -> Result<DriverCloudletHandle> {
        let mut handle = self.handle.lock().unwrap();
        let (resource, store) = Self::get_resource_and_store(&mut handle);
        match self
            .bindings
            .cloudlet_driver_bridge()
            .generic_driver()
            .call_init_cloudlet(
                store,
                resource,
                &cloudlet.name,
                &(&cloudlet.capabilities).into(),
                &(&cloudlet.controller).into(),
            )? {
            Ok(cloudlet) => Ok(Arc::new(WasmCloudlet {
                handle: self.own.clone(),
                resource: cloudlet,
            })),
            Err(error) => Err(anyhow!(error)),
        }
    }
}

impl WasmDriver {
    fn new(
        config: &WasmConfig,
        cloud_identifier: &str,
        name: &str,
        source: &Source,
    ) -> Result<Arc<Self>> {
        let config_directory = Storage::get_config_folder_for_driver(name);
        let data_directory = Storage::get_data_folder_for_driver(name);
        if !config_directory.exists() {
            fs::create_dir_all(&config_directory).unwrap_or_else(|error| {
                warn!(
                    "<red>Failed</> to create configs directory for driver <blue>{}</>: <red>{}</>",
                    &name, &error
                )
            });
        }
        if !data_directory.exists() {
            fs::create_dir_all(&data_directory).unwrap_or_else(|error| {
                warn!(
                    "<red>Failed</> to create data directory for driver <blue>{}</>: <red>{}</>",
                    &name, &error
                )
            });
        }

        let mut engine_config = Config::new();
        engine_config.wasm_component_model(true);
        if let Err(error) =
            engine_config.cache_config_load(Storage::get_configs_folder().join(CONFIG_FILE))
        {
            warn!(
                "<red>Failed</> to enable caching for wasmtime engine: <red>{}</>",
                &error
            );
        }

        let engine = Engine::new(&engine_config)?;
        let component = Component::from_binary(&engine, &source.code)?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)?;
        Driver::add_to_linker(&mut linker, |state: &mut WasmDriverState| state)?;

        let mut wasi = WasiCtxBuilder::new();
        if let Some(config) = config.get_config(name) {
            if config.inherit_stdio {
                wasi.inherit_stdio();
            }
            if config.inherit_args {
                wasi.inherit_args();
            }
            if config.inherit_env {
                wasi.inherit_env();
            }
            if config.inherit_network {
                wasi.inherit_network();
            }
            if config.allow_ip_name_lookup {
                wasi.allow_ip_name_lookup(true);
            }
            for mount in &config.mounts {
                wasi.preopened_dir(&mount.host, &mount.guest, DirPerms::all(), FilePerms::all())?;
            }
        }
        let wasi = wasi
            .preopened_dir(
                &config_directory,
                "/configs/",
                DirPerms::all(),
                FilePerms::all(),
            )?
            .preopened_dir(&data_directory, "/data/", DirPerms::all(), FilePerms::all())?
            .build();

        let table = ResourceTable::new();

        let mut store = Store::new(
            &engine,
            WasmDriverState {
                handle: Weak::new(),
                wasi,
                table,
            },
        );
        let bindings = Driver::instantiate(&mut store, &component, &linker)?;
        let driver = Arc::new_cyclic(|handle| {
            store.data_mut().handle = handle.clone();
            WasmDriver {
                own: handle.clone(),
                name: name.to_string(),
                bindings,
                handle: Mutex::new(None),
                data: WasmDriverData {
                    child_processes: RwLock::new(HashMap::new()),
                },
            }
        });
        let driver_resource = driver
            .bindings
            .cloudlet_driver_bridge()
            .generic_driver()
            .call_constructor(&mut store, cloud_identifier)?;
        driver
            .handle
            .lock()
            .unwrap()
            .replace(WasmDriverHandle::new(store, driver_resource));
        Ok(driver)
    }

    pub fn load_all(
        cloud_identifier: &str,
        drivers: &mut Vec<Arc<dyn GenericDriver>>,
    ) -> WasmConfig {
        // Check if cache configuration exists
        let config_file = Storage::get_configs_folder().join(CONFIG_FILE);
        if !config_file.exists() {
            fs::write(&config_file, DEFAULT_CONFIG).unwrap_or_else(|error| {
                warn!(
                    "<red>Failed</> to create default wasm configuration file: <red>{}</>",
                    &error
                )
            });
        }
        let config = WasmConfig::load_from_file(&config_file).unwrap_or_else(|error| {
            warn!(
                "<red>Failed</> to load wasm configuration file: <red>{}</>",
                &error
            );
            WasmConfig::default()
        });

        let old_loaded = drivers.len();

        let drivers_directory = Storage::get_drivers_folder().join(WASM_DIRECTORY);
        if !drivers_directory.exists() {
            fs::create_dir_all(&drivers_directory).unwrap_or_else(|error| {
                warn!(
                    "<red>Failed</> to create drivers directory: <red>{}</>",
                    &error
                )
            });
        }

        let entries = match fs::read_dir(&drivers_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!(
                    "<red>Failed</> to read driver directory: <red>{}</>",
                    &error
                );
                return config;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("<red>Failed</> to read driver entry: <red>{}</>", &error);
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
                warn!(
                    "The driver directory should only contain wasm files, please remove <blue>{:?}</>",
                    &entry.file_name()
                );
                continue;
            }

            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let source = match Source::from_file(&path) {
                Ok(source) => source,
                Err(error) => {
                    error!(
                        "<red>Failed</> to read source code for driver <blue>{}</> from file(<blue>{:?}</>): <blue>{}</>",
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            info!("Compiling driver <blue>{}</>...", &name);
            let driver = WasmDriver::new(&config, cloud_identifier, &name, &source);
            match driver {
                Ok(driver) => match driver.init() {
                    Ok(info) => {
                        if info.ready {
                            info!(
                                "Loaded driver <blue>{} v{}</> by <blue>{}</>",
                                &driver.name, &info.version,
                                &info.authors.join(", ")
                            );
                            drivers.push(driver);
                        } else {
                            warn!(
                                "Driver <blue>{}</> marked itself as <yellow>not ready</>, skipping...",
                                &driver.name
                            );
                        }
                    }
                    Err(error) => error!("<red>Failed</> to load driver <blue>{}</>: <red>{}</>", &name, &error),
                },
                Err(error) => error!(
                    "<red>Failed</> to compile driver <blue>{}</> at location <blue>{}</>: <red>{}</>",
                    &name,
                    &source,
                    &error
                ),
            }
        }

        if old_loaded == drivers.len() {
            warn!(
                "The Wasm driver feature is <yellow>enabled</>, but no Wasm drivers were loaded."
            );
        }
        config
    }
}

impl From<bridge::Information> for Information {
    fn from(val: bridge::Information) -> Self {
        Information {
            authors: val.authors,
            version: val.version,
            ready: val.ready,
        }
    }
}

impl From<&Capabilities> for bridge::Capabilities {
    fn from(val: &Capabilities) -> Self {
        bridge::Capabilities {
            memory: val.memory,
            max_allocations: val.max_allocations,
            child: val.child.clone(),
        }
    }
}

impl From<&RemoteController> for bridge::RemoteController {
    fn from(val: &RemoteController) -> Self {
        bridge::RemoteController {
            address: val.address.to_string(),
        }
    }
}

impl From<&HostAndPort> for bridge::Address {
    fn from(val: &HostAndPort) -> Self {
        bridge::Address {
            host: val.host.clone(),
            port: val.port,
        }
    }
}
