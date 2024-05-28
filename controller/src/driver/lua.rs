use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::exit;
use std::sync::Arc;
use ::log::{error, info, Level, warn};

use colored::Colorize;
use mlua::{Function, Lua, LuaSerdeExt, Table};

use crate::driver::lua::log::set_log_function;
use crate::driver::{Driver, DRIVERS_DIRECTORY, Information};
use crate::driver::source::Source;
use crate::node::Node;
use crate::VERSION;

const LUA_DIRECTORY: &str = "lua";
const DRIVER_MAIN_FILE: &str = "driver.lua";

pub struct LuaDriver {
    pub name: String,
    lua_runtime: Lua,
}

impl Driver for LuaDriver {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn init(&self) -> Result<Information, Box<dyn Error>> {
        let init: Function = self.lua_runtime.globals().get("Init")?;
        let result = init.call(())?;
        Ok(result)
    }

    fn init_node(&self, node: &Node) -> Result<bool, Box<dyn Error>> {
        let init: Function = self.lua_runtime.globals().get("InitNode")?;
        let result = init.call(self.lua_runtime.to_value(node)?)?;
        Ok(result)
    }

    fn stop_server(&self, server: &str) -> Result<(), Box<dyn Error>> {
        let stop: Function = self.lua_runtime.globals().get("StopServer")?;
        stop.call::<_, ()>(server)?;
        Ok(())
    }

    fn start_server(&self, server: &str) -> Result<(), Box<dyn Error>> {
        let start: Function = self.lua_runtime.globals().get("StartServer")?;
        start.call::<_, ()>(server)?;
        Ok(())
    }
}

impl LuaDriver {
    fn new(name: &str, source: &Source) -> Self {
        let lua = Lua::new();

        set_builtin(&lua);

        set_log_function(&lua, "print", Level::Info);
        set_log_function(&lua, "warn", Level::Warn);
        set_log_function(&lua, "error", Level::Error);
        set_log_function(&lua, "debug", Level::Debug);

        let chunk = lua.load(&source.code);
        chunk.exec().unwrap_or_else(|error| {
            error!(
                "Failed to compile Lua source code({:?}): {}",
                &source.path.file_name().unwrap(),
                &error
            );
            exit(1);
        });

        Self {
            name: name.to_owned(),
            lua_runtime: lua,
        }
    }

    pub fn load_drivers(drivers: &mut Vec<Arc<dyn Driver>>) {
        let old_loaded = drivers.len();

        let drivers_directory = Path::new(DRIVERS_DIRECTORY).join(LUA_DIRECTORY);
        if !drivers_directory.exists() {
            fs::create_dir_all(&drivers_directory).unwrap_or_else(|error| {
                warn!("{} to create Lua drivers directory: {}", "Failed".red(), &error)
            });
        }

        let entries = match fs::read_dir(&drivers_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read Lua driver directory: {}", "Failed".red(), &error);
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read Lua driver entry: {}", "Failed".red(), &error);
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_dir() {
                warn!(
                    "The driver directory should only contain folders, please remove {:?}",
                    &entry.file_name()
                );
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
                    error!(
                        "{} to read source code for Lua driver {} from file({:?}): {}",
                        "Failed".red(),
                        &name,
                        &driver_entry,
                        &error
                    );
                    continue;
                }
            };

            let driver = LuaDriver::new(&name, &source);
            match driver.init() {
                Ok(info) => {
                    info!(
                        "Loaded Lua driver {} by {}",
                        format!("{} v{}", &driver.name, &info.version).blue(),
                        &info.author.blue()
                    );
                    drivers.push(Arc::new(driver));
                }
                Err(error) => error!(
                    "{} to load Lua driver {}: {}",
                    "Failed".red(),
                    &name,
                    &error
                ),
            }
        }

        if old_loaded == drivers.len() {
            warn!("The Lua driver feature is enabled, but no Lua drivers were loaded.");
        }
    }
}

fn set_builtin(lua: &Lua) {
    let globals = lua.globals();

    let table = lua.create_table().expect("Failed to create table");
    set_builtin_data(lua, &table);
    set_builtin_functions(lua, &table);

    globals.set("builtin", table).expect("Failed to set builtin table");
}

fn set_builtin_data(_lua: &Lua, table: &Table) {
    table.set("version", VERSION.to_string()).unwrap();
}

fn set_builtin_functions(_lua: &Lua, _table: &Table) {
    // Add any required functions here
}

mod log {
    use log::{log, Level};
    use mlua::Lua;

    pub fn set_log_function(lua: &Lua, name: &str, level: Level) {
        lua.globals()
            .set(
                name,
                lua.create_function(move |_, message: String| {
                    log!(level, "{}", &message);
                    Ok(())
                })
                    .unwrap(),
            )
            .unwrap();
    }
}

mod compat {
    use mlua::{FromLua, Lua, LuaSerdeExt, Value};
    use crate::driver::Information;

    impl FromLua<'_> for Information {
        fn from_lua(value: Value<'_>, lua: &'_ Lua) -> mlua::Result<Self> {
            lua.from_value(value)
        }
    }
}