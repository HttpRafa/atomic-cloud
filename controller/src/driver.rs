use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;
use log::{error, info, warn};
use mlua::{FromLua, Lua, LuaSerdeExt, Value};
use serde::Deserialize;
use crate::driver::lua::LuaDriver;

pub mod lua;
mod http;

pub const DRIVER_DIRECTORY: &str = "drivers";
const DRIVER_MAIN_FILE: &str = "driver.lua";

pub struct Drivers {
    drivers: Vec<Arc<LuaDriver>>
}

impl Drivers {
    pub fn new() -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();
        for dir in fs::read_dir(DRIVER_DIRECTORY).expect("Failed to read driver directory") {
            match dir {
                Ok(dir) => {
                    let path = &dir.path();
                    if path.is_dir() {
                        let driver_entry = Path::new(&path).join(DRIVER_MAIN_FILE);
                        if driver_entry.exists() {
                            let name = String::from(dir.file_name().to_string_lossy());
                            info!("Loading driver {}...", name);
                            let driver = LuaDriver::new(Source::from_file(driver_entry));
                            let info = driver.init();
                            match info {
                                Ok(info) => {
                                    info!("Loaded driver {} v{} by {}", name, info.version, info.author);
                                    drivers.push(Arc::new(driver));
                                }
                                Err(error) => error!("Failed to load driver {}: {}", name, error)
                            }
                        }
                    } else {
                        warn!("The driver directory should only contain folders please remove {:?}", dir.file_name());
                    }
                }
                Err(error) => error!("Failed to read driver: {}", error)
            }
        }

        info!("Loaded {} driver[s]", drivers.len());
        Drivers {
            drivers
        }
    }
}

#[derive(Deserialize)]
pub struct Information {
    author: String,
    version: String
}

impl FromLua<'_> for Information {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

pub(crate) struct Source {
    path: PathBuf,
    code: String
}

impl Source {
    fn from_file(path: PathBuf) -> Self {
        let code = fs::read_to_string(&path).unwrap_or_else(|error| {
            error!("Failed to read source code from file({:?}): {}", path, error);
            exit(1);
        });
        Source {
            path,
            code
        }
    }
}