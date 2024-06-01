use std::fs;
use std::path::Path;
use std::sync::{Arc, Weak};

use anyhow::Result;
use colored::Colorize;
use exports::node::driver::bridge;
use inquire::{required, Password, Text};
use log::{debug, error, info, warn};
use node::driver;
use node::driver::http::{Header, Method, Response};
use node::driver::log::{Level, Question};
use tokio::sync::Mutex;
use tonic::async_trait;
use wasmtime::component::{bindgen, Component, Linker, ResourceAny};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

use crate::config::auto_complete::SimpleAutoComplete;
use crate::config::validators::UnsignedValidator;
use crate::config::CONFIG_DIRECTORY;
use crate::driver::{DRIVERS_DIRECTORY, GenericDriver, Information};
use crate::driver::source::Source;
use crate::node::{Capability, Node};

use super::DATA_DIRECTORY;

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
    async fn log_string(&mut self, level: Level, message: String) {
        match level {
            Level::Info => info!("{}", message),
            Level::Warn => warn!("{}", message),
            Level::Error => error!("{}", message),
            Level::Debug => debug!("{}", message),
        }
    }
    async fn request_user_input(&mut self, question: Question, message: String, auto_complete: Vec<String>) -> Option<String> {
        let driver = self.handle.upgrade().unwrap();
        match question {
            Question::Text => {
                let result = Text::new(&message).with_validator(required!()).with_autocomplete(SimpleAutoComplete::from_strings(auto_complete)).prompt_skippable();
                if result.is_ok() {
                    return result.unwrap();
                }
                warn!("{} to get user input for question asked by driver {}: {}", "Failed".red(), &driver.name.blue(), result.unwrap_err());
                None
            },
            Question::Password => {
                let result = Password::new(&message).with_validator(required!()).prompt_skippable();
                if result.is_ok() {
                    return result.unwrap();
                }
                warn!("{} to get user input for question asked by driver {}: {}", "Failed".red(), &driver.name.blue(), result.unwrap_err());
                None
            },
            Question::UnsignedNumber => {
                let result = Text::new(&message).with_validator(required!()).with_validator(UnsignedValidator::new()).with_autocomplete(SimpleAutoComplete::from_strings(auto_complete)).prompt_skippable();
                if result.is_ok() {
                    return result.unwrap();
                }
                warn!("{} to get user input for question asked by driver {}: {}", "Failed".red(), &driver.name.blue(), result.unwrap_err());
                None
            }
        }
    }
}

#[async_trait]
impl driver::http::Host for WasmDriverState {
    async fn send_http_request(&mut self, method: Method, url: String, headers: Vec<Header>) -> Option<Response> {
        let driver = self.handle.upgrade().unwrap();
        let mut request = match method {
            Method::Get => minreq::get(url),
            Method::Post => minreq::post(url),
            Method::Put => minreq::put(url),
            Method::Delete => minreq::delete(url),
        };
        for header in headers {
            request = request.with_header(&header.key, &header.value);
        }
        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                warn!("{} to send HTTP request for driver {}: {}", "Failed".red(), &driver.name.blue(), error);
                return None;
            }
        };
        Some(Response {
            status_code: response.status_code as u32,
            reason_phrase: response.reason_phrase.clone(),
            headers: response.headers.iter().map(|header| Header { key: header.0.clone(), value: header.1.clone() }).collect(),
            bytes: response.into_bytes(),
        })
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

pub struct WasmDriver {
    pub name: String,
    bindings: Driver,
    handle: Mutex<WasmDriverHandle>,
}

#[async_trait]
impl GenericDriver for WasmDriver {
    fn name(&self) -> &String {
        &self.name
    }

    async fn init(&self) -> Result<Information> {
        let mut handle = self.handle.lock().await;
        let (resource, store) = handle.get();
        match self.bindings.node_driver_bridge().generic_driver().call_init(store, resource).await {
            Ok(information) => Ok(information.into()),
            Err(error) => Err(error),
        }
    }

    async fn init_node(&self, node: &Node) -> Result<bool> {
        let mut handle = self.handle.lock().await;
        let (resource, store) = handle.get();
        self.bindings.node_driver_bridge().generic_driver().call_init_node(store, resource, &node.name, &node.capabilities.iter().map(|cap| cap.into()).collect::<Vec<bridge::Capability>>()).await
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
        let config_directory = Path::new(CONFIG_DIRECTORY).join(name);
        let data_directory = Path::new(DRIVERS_DIRECTORY).join(DATA_DIRECTORY).join(name);
        if !config_directory.exists() {
            fs::create_dir_all(&config_directory).unwrap_or_else(|error| {
                warn!("{} to create configs directory for driver {}: {}", "Failed".red(), &name, &error)
            });
        }
        if !data_directory.exists() {
            fs::create_dir_all(&data_directory).unwrap_or_else(|error| {
                warn!("{} to create data directory for driver {}: {}", "Failed".red(), &name, &error)
            });
        }

        let mut config = Config::new();
        config.async_support(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, &source.code)?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        Driver::add_to_linker(&mut linker, |state: &mut WasmDriverState| state)?;

        let wasi = WasiCtxBuilder::new().inherit_stdio().preopened_dir(&config_directory, "/configs/", DirPerms::all(), FilePerms::all())?.preopened_dir(&data_directory, "/data/", DirPerms::all(), FilePerms::all())?.build();
        let table = ResourceTable::new();

        let mut store = Store::new(&engine, WasmDriverState {
            handle: Weak::new(),
            wasi,
            table,
        });
        let (bindings, _) = Driver::instantiate_async(&mut store, &component, &linker).await?;
        let driver_resource = bindings.node_driver_bridge().generic_driver().call_constructor(&mut store).await?;
        Ok(Arc::new_cyclic(|handle| {
            store.data_mut().handle = handle.clone();
            WasmDriver {
                name: name.to_string(),
                bindings,
                handle: Mutex::new(WasmDriverHandle::new(store, driver_resource)),
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

            info!("Compiling driver {}...", &name.blue());
            let driver = WasmDriver::new(&name, &source).await;
            match driver {
                Ok(driver) => match driver.init().await {
                    Ok(info) => {
                        if info.ready {
                            info!(
                                "Loaded driver {} by {}",
                                format!("{} v{}", &driver.name, &info.version).blue(),
                                &info.authors.join(", ").blue()
                            );
                            drivers.push(driver);
                        } else {
                            warn!("Driver {} marked itself as {}, skipping...", &driver.name.blue(), "not ready".yellow());
                        }
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

impl From<bridge::Information> for Information {
    fn from(val: bridge::Information) -> Self {
        Information {
            authors: val.authors,
            version: val.version,
            ready: val.ready,
        }
    }
}

impl From<&Capability> for bridge::Capability {
    fn from(val: &Capability) -> Self {
        match val {
            Capability::LimitedMemory(memory) => bridge::Capability::LimitedMemory(*memory),
            Capability::UnlimitedMemory(enabled) => bridge::Capability::UnlimitedMemory(*enabled),
            Capability::MaxServers(servers) => bridge::Capability::MaxServers(*servers),
            Capability::SubNode(node) => bridge::Capability::SubNode(node.to_owned()),
        }
    }
}