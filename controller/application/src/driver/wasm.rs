use std::fs;
use std::path::Path;
use std::sync::{Arc, Weak};

use anyhow::Result;
use colored::Colorize;
use exports::node::driver::bridge::Information;
use log::{debug, error, info, warn};
use node::driver;
use tokio::sync::Mutex;
use tonic::async_trait;
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::driver::{DRIVERS_DIRECTORY, GenericDriver, DriverInformation};
use crate::driver::source::Source;
use crate::node::Node;

bindgen!({
    world: "driver",
    path: "../structure/wit/",
    async: true,
});

const WASM_DIRECTORY: &str = "wasm";

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

#[async_trait]
impl driver::api::Host for WasmDriverState {
    async fn get_name(&mut self) -> String {
        self.handle.upgrade().unwrap().name.clone()
    }
}

#[async_trait]
impl driver::log::Host for WasmDriverState {
    async fn info(&mut self, message: String) {
        info!("{}", &message);
    }
    async fn warn(&mut self, message: String) {
        warn!("{}", &message);
    }
    async fn error(&mut self, message: String) {
        error!("{}", &message);
    }
    async fn debug(&mut self, message: String) {
        debug!("{}", &message);
    }
}

struct WasmDriverHandle {
    store: Store<WasmDriverState>
}

impl WasmDriverHandle {
    fn new(store: Store<WasmDriverState>) -> Self {
        WasmDriverHandle { store }
    }

    fn store(&mut self) -> &mut Store<WasmDriverState> {
        &mut self.store
    }
}

impl Information {
    fn to_generic(self) -> DriverInformation {
        DriverInformation {
            authors: self.authors,
            version: self.version,
        }
    }
}

pub struct WasmDriver {
    pub name: String,
    bindings: Driver,
    handle: Mutex<WasmDriverHandle>,
}

#[async_trait]
impl GenericDriver for WasmDriver {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn init(&self) -> Result<DriverInformation> {
        let mut handle = self.handle.lock().await;
        match self.bindings.node_driver_bridge().call_init(handle.store()).await {
            Ok(information) => Ok(information.to_generic()),
            Err(error) => Err(error),
        }
    }

    async fn init_node(&self, _node: &Node) -> Result<bool> {
        todo!()
    }

    async fn stop_server(&self, _server: &str) -> Result<()> {
        todo!()
    }

    async fn start_server(&self, _server: &str) -> Result<()> {
        todo!()
    }
}

impl WasmDriver {
    async fn new(name: &str, source: &Source) -> Result<Arc<Self>> {
        let mut config = Config::new();
        config.async_support(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, &source.code)?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        Driver::add_to_linker(&mut linker, |state: &mut WasmDriverState| state)?;

        let wasi = WasiCtxBuilder::new().build();
        let table = ResourceTable::new();

        let mut store = Store::new(&engine, WasmDriverState {
            handle: Weak::new(),
            wasi,
            table,
        });
        let (bindings, _) = Driver::instantiate_async(&mut store, &component, &linker).await?;
        Ok(Arc::new_cyclic(|handle| {
            store.data_mut().handle = handle.clone();
            WasmDriver {
                name: name.to_string(),
                bindings,
                handle: Mutex::new(WasmDriverHandle::new(store)),
            }
        }))
    }

    pub async fn load_all(drivers: &mut Vec<Arc<dyn GenericDriver>>) {
        let old_loaded = drivers.len();

        let drivers_directory = Path::new(DRIVERS_DIRECTORY).join(WASM_DIRECTORY);
        if !drivers_directory.exists() {
            fs::create_dir_all(&drivers_directory).unwrap_or_else(|error| {
                warn!("{} to create drivers directory: {}", "Failed".red(), &error)
            });
        }

        let entries = match fs::read_dir(&drivers_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read driver directory: {}", "Failed".red(), &error);
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read driver entry: {}", "Failed".red(), &error);
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() || !path.file_name().unwrap().to_string_lossy().ends_with(".wasm") {
                warn!(
                    "The driver directory should only contain wasm files, please remove {:?}",
                    &entry.file_name()
                );
                continue;
            }

            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let source = match Source::from_file(&path) {
                Ok(source) => source,
                Err(error) => {
                    error!(
                        "{} to read source code for driver {} from file({:?}): {}",
                        "Failed".red(),
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            let driver = WasmDriver::new(&name, &source).await;
            match driver {
                Ok(driver) => match driver.init().await {
                    Ok(info) => {
                        info!(
                            "Loaded driver {} by {}",
                            format!("{} v{}", &driver.name, &info.version).blue(),
                            &info.authors.join(", ").blue()
                        );
                        drivers.push(driver);
                    }
                    Err(error) => error!(
                        "{} to load driver {}: {}",
                        "Failed".red(),
                        &name,
                        &error
                    ),
                },
                Err(error) => error!(
                    "{} to compile driver {}: {}",
                    "Failed".red(),
                    &name,
                    &error
                ),
            }
        }

        if old_loaded == drivers.len() {
            warn!("The Wasm driver feature is enabled, but no Wasm drivers were loaded.");
        }
    }
}