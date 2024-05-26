use std::process::exit;

use ::log::{error, Level};
use mlua::{Function, Lua};
use crate::driver::http::set_http_functions;

use crate::driver::lua::log::set_log_function;
use crate::driver::{Information, Source};
use crate::VERSION;

pub struct LuaDriver {
    lua_runtime: Lua,
}

impl LuaDriver {
    pub fn new(source: Source) -> Self {
        let lua = Lua::new();

        set_global_data(&lua);

        set_log_function(&lua, "print", Level::Info);
        set_log_function(&lua, "warn", Level::Warn);
        set_log_function(&lua, "error", Level::Error);
        set_log_function(&lua, "debug", Level::Debug);

        set_http_functions(&lua);

        let chunk = lua.load(&source.code);
        chunk.exec().unwrap_or_else(|error| {
            error!("Failed to compile driver source code({:?}): {}", &source.path.file_name().as_ref().unwrap(), error);
            exit(1);
        });

        LuaDriver {
            lua_runtime: lua
        }
    }

    pub fn init(&self) -> Result<Information, mlua::Error> {
        let init: Function = self.lua_runtime.globals().get("Init")?;
        let result = init.call(())?;
        Ok(result)
    }

    pub async fn stop_server(&self, server: &str) -> Result<(), mlua::Error> {
        let init: Function = self.lua_runtime.globals().get("StopServer")?;
        init.call_async::<_, ()>(server).await?;
        Ok(())
    }

    pub async fn start_server(&self, server: &str) -> Result<(), mlua::Error> {
        let init: Function = self.lua_runtime.globals().get("StartServer")?;
        init.call_async::<_, ()>(server).await?;
        Ok(())
    }
}

fn set_global_data(lua: &Lua) {
    let globals = &lua.globals();

    let table = lua.create_table().unwrap();
    {
        table.set("version", format!("{}", VERSION)).unwrap();
    }

    globals.set("mod", table).unwrap();
}

mod log {
    use log::{Level, log};
    use mlua::Lua;

    pub fn set_log_function(lua: &Lua, name: &str, level: Level) {
        lua.globals().set(name, lua.create_function(move |_, message: String| {
            log!(level, "{}", message);
            Ok(())
        }).unwrap()).unwrap();
    }
}