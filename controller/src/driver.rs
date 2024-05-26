use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use colored::Colorize;
use log::{error, info, warn};
use mlua::{FromLua, Lua, LuaSerdeExt, Value};
use serde::Deserialize;
use crate::driver::lua::LuaDriver;

pub mod lua;
mod http;

pub(crate) const DRIVERS_DIRECTORY: &str = "drivers";
const DRIVER_MAIN_FILE: &str = "driver.lua";

pub struct Drivers {
    drivers: Vec<Arc<LuaDriver>>,
}

impl Drivers {
    pub fn load_all() -> Self {
        info!("Loading drivers...");

        let mut drivers = Vec::new();
        let entries = match fs::read_dir(DRIVERS_DIRECTORY) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read driver directory: {}", "Failed".red(), &error);
                return Drivers { drivers };
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
            if !path.is_dir() {
                warn!("The driver directory should only contain folders, please remove {:?}", &entry.file_name());
                continue;
            }

            let driver_entry = path.join(DRIVER_MAIN_FILE);
            if !driver_entry.exists() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let source = match Source::from_file(&driver_entry) {
                Ok(source) => source,
                Err(error) => {
                    error!("{} to read source code for driver {} from file({:?}): {}", "Failed".red(), &name, &driver_entry, &error);
                    continue;
                }
            };

            let driver = LuaDriver::new(&name, &source);
            match driver.init() {
                Ok(info) => {
                    info!("Loaded driver {} by {}", format!("{} v{}", &driver.name, &info.version).blue(), &info.author.blue());
                    drivers.push(Arc::new(driver));
                }
                Err(error) => error!("{} to load driver {}: {}", "Failed".red(), &name, &error),
            }
        }

        info!("Loaded {}", format!("{} driver(s)", drivers.len()).blue());
        Drivers { drivers }
    }
    pub fn find_by_name(&self, name: &String) -> Option<Arc<LuaDriver>> {
        for driver in &self.drivers {
            if driver.name.eq_ignore_ascii_case(&name) {
                return Some(Arc::clone(driver));
            }
        }
        None
    }
}

#[derive(Deserialize)]
pub struct Information {
    author: String,
    version: String,
}

impl FromLua<'_> for Information {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

pub(crate) struct Source {
    path: PathBuf,
    code: String,
}

impl Source {
    fn from_file(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let path = path.to_owned();
        let code = fs::read_to_string(&path)?;
        Ok(Source { path, code })
    }
}