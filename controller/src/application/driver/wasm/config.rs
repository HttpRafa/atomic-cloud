use common::config::{LoadFromTomlFile, SaveToTomlFile};
use regex::Regex;
use serde::{Deserialize, Serialize};
use simplelog::warn;

#[derive(Serialize, Deserialize, Default)]
pub struct WasmConfig {
    pub drivers: Vec<DriverConfig>,
}

impl LoadFromTomlFile for WasmConfig {}
impl SaveToTomlFile for WasmConfig {}

impl WasmConfig {
    pub fn get_config(&self, name: &str) -> Option<&DriverConfig> {
        self.drivers
            .iter()
            .find(|driver| match Regex::new(&driver.name) {
                Ok(regex) => regex.is_match(name),
                Err(error) => {
                    warn!("Failed to compile driver name regex: {}", error);
                    false
                }
            })
    }
}

#[derive(Serialize, Deserialize)]
pub struct DriverConfig {
    pub name: String,
    pub inherit_stdio: bool,
    pub inherit_args: bool,
    pub inherit_env: bool,
    pub inherit_network: bool,
    pub allow_ip_name_lookup: bool,
    pub allow_http: bool,
    pub allow_process: bool,
    pub allow_remove_dir_all: bool,

    pub mounts: Vec<MountConfig>,
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            inherit_stdio: true,
            inherit_args: true,
            inherit_env: true,
            inherit_network: true,
            allow_ip_name_lookup: true,
            allow_http: true,
            allow_process: true,
            allow_remove_dir_all: true,
            mounts: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MountConfig {
    pub host: String,
    pub guest: String,
}
