use std::fs;
use std::path::Path;
use log::{info, Level, log};
use mlua::{Function, Lua};
use crate::config::Config;
use crate::VERSION;

pub const DRIVER_DIRECTORY: &str = "drivers";

pub struct Driver {
    lua_runtime: Lua,
}

impl Driver {
    fn new(code: Vec<Source>) -> Self {
        let lua = Lua::new();

        Self::set_log_function(&lua, "print", Level::Info);
        Self::set_log_function(&lua, "warn", Level::Warn);
        Self::set_log_function(&lua, "error", Level::Error);
        Self::set_log_function(&lua, "debug", Level::Debug);

        lua.globals().set("controller_version", VERSION).unwrap();

        for file in code {
            let chunk = lua.load(&file.code);
            if chunk.exec().is_err() {
                panic!("Failed to compile driver source code. File: {}", file.file_name);
            }
        }

        Driver {
            lua_runtime: lua
        }
    }

    pub fn init(&self) -> Result<(), mlua::Error> {
        let init: Function = self.lua_runtime.globals().get("init")?;
        init.call(())?;
        Ok(())
    }

    pub async fn start_server(&self, server: &str) -> Result<(), mlua::Error> {
        let init: Function = self.lua_runtime.globals().get("start_server")?;
        init.call_async::<_, ()>(server).await?;
        Ok(())
    }

    fn set_log_function(lua: &Lua, name: &str, level: Level) {
        lua.globals().set(name, lua.create_function(move |_, message: String| {
            log!(level, "{}", message);
            Ok(())
        }).expect("Failed to set log function")).expect("Failed to set log function");
    }
}

struct Source {
    file_name: String,
    code: String
}

pub fn load_server_driver(config: &Config) -> Driver {
    info!("Loading driver system...");

    let code = fs::read_dir(Path::new(DRIVER_DIRECTORY)
        .join(config.driver.as_ref().unwrap()))
        .expect("Failed to read code of driver")
        .filter_map(Result::ok)
        .map(|entry| Source {
            file_name: entry.file_name().to_string_lossy().to_string(),
            code: fs::read_to_string(entry.path()).unwrap()
        })
        .collect::<Vec<_>>();

    let driver = Driver::new(code);
    driver.init().expect("Failed to initialize driver");
    driver
}