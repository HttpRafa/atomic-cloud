use std::path::PathBuf;

use anyhow::Result;
use bitflags::bitflags;
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

bitflags! {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(transparent)]
    pub struct Permissions: u32 {
        const INHERIT_STDIO = 1;
        const INHERIT_ARGS = 1 << 1;
        const INHERIT_ENV = 1 << 2;
        const INHERIT_NETWORK = 1 << 3;
        const ALLOW_IP_NAME_LOOKUP = 1 << 4;
        const ALLOW_HTTP = 1 << 5;
        const ALLOW_PROCESS = 1 << 6;
        const ALLOW_REMOVE_DIR_ALL = 1 << 7;
        const ALL = Self::INHERIT_STDIO.bits() | Self::INHERIT_ARGS.bits() | Self::INHERIT_ENV.bits() | Self::INHERIT_NETWORK.bits() | Self::ALLOW_IP_NAME_LOOKUP.bits() | Self::ALLOW_HTTP.bits() | Self::ALLOW_PROCESS.bits() | Self::ALLOW_REMOVE_DIR_ALL.bits();
    }
}

#[derive(Serialize, Deserialize)]
pub struct PluginConfig {
    name: String,
    permissions: Permissions,

    mounts: Vec<Mount>,
}

impl PluginConfig {
    pub fn get_permissions(&self) -> &Permissions {
        &self.permissions
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
