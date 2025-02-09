use std::path::PathBuf;

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use simplelog::warn;
use tokio::fs;

use crate::storage::{LoadFromTomlFile, Storage};

const DEFAULT_PLUGINS_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/configs/wasm-plugins.toml"
));
const DEFAULT_ENGINE_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/configs/wasm-engine.toml"
));

#[derive(Serialize, Deserialize)]
pub struct PluginsConfig {
    plugins: Vec<PluginConfig>,
}

#[allow(
    clippy::struct_excessive_bools,
    reason = "Mybe refactor this in the future to use bitflags"
)]
#[derive(Serialize, Deserialize)]
pub struct PluginConfig {
    name: String,
    inherit_stdio: bool,
    inherit_args: bool,
    inherit_env: bool,
    inherit_network: bool,
    allow_ip_name_lookup: bool,
    allow_http: bool,
    allow_process: bool,
    allow_remove_dir_all: bool,

    mounts: Vec<Mount>,
}

impl PluginConfig {
    pub fn has_inherit_stdio(&self) -> bool {
        self.inherit_stdio
    }
    pub fn has_inherit_args(&self) -> bool {
        self.inherit_args
    }
    pub fn has_inherit_env(&self) -> bool {
        self.inherit_env
    }
    pub fn has_inherit_network(&self) -> bool {
        self.inherit_network
    }
    pub fn has_allow_ip_name_lookup(&self) -> bool {
        self.allow_ip_name_lookup
    }
    pub fn has_allow_http(&self) -> bool {
        self.allow_http
    }
    pub fn has_allow_process(&self) -> bool {
        self.allow_process
    }
    pub fn has_allow_remove_dir_all(&self) -> bool {
        self.allow_remove_dir_all
    }
    pub fn get_mounts(&self) -> &[Mount] {
        &self.mounts
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mount {
    host: String,
    guest: String,
}

impl Mount {
    pub fn get_host(&self) -> &str {
        &self.host
    }
    pub fn get_guest(&self) -> &str {
        &self.guest
    }
}

impl PluginsConfig {
    pub async fn parse() -> Result<Self> {
        let path = Storage::wasm_plugins_config_file();
        if path.exists() {
            Self::from_file(&path).await
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&path, DEFAULT_PLUGINS_CONFIG).await?;
            Self::from_file(&path).await
        }
    }

    pub fn find_config(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins
            .iter()
            .find(|plugin| match Regex::new(&plugin.name) {
                Ok(regex) => regex.is_match(name),
                Err(error) => {
                    warn!("Failed to compile plugin name regex: {}", error);
                    false
                }
            })
    }
}

pub async fn verify_engine_config() -> Result<PathBuf> {
    let path = Storage::wasm_engine_config_file();
    if path.exists() {
        Ok(path)
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, DEFAULT_ENGINE_CONFIG).await?;
        Ok(path)
    }
}

impl LoadFromTomlFile for PluginsConfig {}
