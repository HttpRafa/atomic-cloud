pub mod lua;
mod http;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use log::{error, info};
use crate::config::Config;
use crate::driver::lua::LuaDriver;

pub const DRIVER_DIRECTORY: &str = "drivers";
const DRIVER_MAIN_FILE: &str = "driver.lua";

pub struct Source {
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

pub async fn load_server_driver(config: &Config) -> LuaDriver {
    info!("Loading driver system...");

    let code_path = Path::new(DRIVER_DIRECTORY).join(config.driver.as_ref().unwrap()).join(DRIVER_MAIN_FILE);
    if !code_path.exists() {
        error!("Failed to locate driver source code in location {:?}", code_path);
        exit(1);
    }

    // For now, we only support drivers written in Lua
    let driver = LuaDriver::new(Source::from_file(code_path));
    driver.init().unwrap_or_else(|error| {
        error!("Failed to call init of driver {}: {}", &config.driver.as_ref().unwrap(), error);
        exit(1);
    });
    driver.stop_server("stopServer").await.unwrap_or_else(|error| {
        error!("Failed to call stopServer of driver {}: {}", &config.driver.as_ref().unwrap(), error);
        exit(1);
    });
    driver.start_server("startServer").await.unwrap_or_else(|error| {
        error!("Failed to call startServer of driver {}: {}", &config.driver.as_ref().unwrap(), error);
        exit(1);
    });
    driver
}