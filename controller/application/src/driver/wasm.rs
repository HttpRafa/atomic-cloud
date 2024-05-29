use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Weak};

use colored::Colorize;
use log::{error, info, warn};
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Config, Engine, Store};

use crate::driver::{DRIVERS_DIRECTORY, GenericDriver, Information};
use crate::driver::source::Source;
use crate::node::Node;

bindgen!({
    world: "driver",
    path: "../structure/wit/"
});

const WASM_DIRECTORY: &str = "wasm";

struct WasmDriverState {
    handle: Weak<WasmDriver>,
}

impl DriverImports for WasmDriverState {
    fn info(&mut self, message: String) {
        info!("{}", &message);
    }

    fn name(&mut self) -> String {
        self.handle.upgrade().unwrap().name.clone()
    }
}

pub struct WasmDriver {
    pub name: String,
    bindings: Driver,
    store: Store<WasmDriverState>,
}

impl GenericDriver for WasmDriver {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn init(&self) -> Result<Information, Box<dyn Error>> {
        Ok(Information {
            author: "".to_string(),
            version: "".to_string(),
        })
    }

    fn init_node(&self, _node: &Node) -> Result<bool, Box<dyn Error>> {
        todo!()
    }

    fn stop_server(&self, _server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn start_server(&self, _server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

impl WasmDriver {
    fn new(name: &str, source: &Source) -> Result<Arc<Self>, Box<dyn Error>> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        let component = Component::from_binary(&engine, &source.code)?;

        let mut linker = Linker::new(&engine);
        Driver::add_to_linker(&mut linker, |state: &mut WasmDriverState| state)?;

        let mut store = Store::new(&engine, WasmDriverState {
            handle: Weak::new(),
        });
        let (bindings, _) = Driver::instantiate(&mut store, &component, &linker)?;
        Ok(Arc::new_cyclic(|handle| {
            store.data_mut().handle = handle.clone();
            WasmDriver {
                name: name.to_string(),
                bindings,
                store,
            }
        }))
    }

    pub fn load_all(drivers: &mut Vec<Arc<dyn GenericDriver>>) {
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

            let driver = WasmDriver::new(&name, &source);
            match driver {
                Ok(driver) => match driver.init() {
                    Ok(info) => {
                        info!(
                            "Loaded driver {} by {}",
                            format!("{} v{}", &driver.name, &info.version).blue(),
                            &info.author.blue()
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