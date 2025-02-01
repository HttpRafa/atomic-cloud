use std::{fs, path::PathBuf};

use anyhow::Result;
use common::config::{LoadFromTomlFile};
use regex::Regex;
use serde::{Deserialize, Serialize};
use simplelog::warn;

use crate::storage::Storage;

const DEFAULT_PLUGINS_CONFIG: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/wasm-plugins.toml"));
const DEFAULT_ENGINE_CONFIG: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/wasm-engine.toml"));

#[derive(Serialize, Deserialize)]
pub struct PluginsConfig {
    plugins: Vec<PluginConfig>,
}

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

#[derive(Serialize, Deserialize)]
pub struct Mount {
    host: String,
    guest: String,
}

impl PluginsConfig {
    pub fn parse() -> Result<Self> {
        let path = Storage::get_wasm_plugins_config_file();
        if path.exists() {
            Self::from_file(&path)
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, DEFAULT_PLUGINS_CONFIG)?;
            Self::from_file(&path)
        }
    }

    pub fn find_config(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins
            .iter()
            .find(|plugin| match Regex::new(&plugin.name) {
                Ok(regex) => regex.is_match(name),
                Err(error) => {
                    warn!("Failed to compile driver name regex: {}", error);
                    false
                }
            })
    }
}

pub fn verify_engine_config() -> Result<PathBuf> {
    let path = Storage::get_wasm_engine_config_file();
    if path.exists() {
        Ok(path)
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, DEFAULT_ENGINE_CONFIG)?;
        Ok(path)
    }
}

impl Default for PluginConfig {
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

impl LoadFromTomlFile for PluginsConfig {}