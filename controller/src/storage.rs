/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::{borrow::Cow, path::{Path, PathBuf}};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use simplelog::warn;
use tokio::fs;

/* Logs */
const LOGS_DIRECTORY: &str = "logs";
const LATEST_LOG_FILE: &str = "latest.log";

/* Nodes */
const NODES_DIRECTORY: &str = "nodes";

/* Groups */
const GROUPS_DIRECTORY: &str = "groups";

/* Auth */
const USERS_DIRECTORY: &str = "users";

/* Configs */
const CONFIG_DIRECTORY: &str = "configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

/* Wasm Configs */
const WASM_PLUGINS_CONFIG_FILE: &str = "wasm-plugins.toml";
const WASM_ENGINE_CONFIG_FILE: &str = "wasm-engine.toml";

/* Certificates */
const CERT_DIRECTORY: &str = "certs";
const CERT_FILE: &str = "certificate.crt";
const CERT_PRIVATE_KEY_FILE: &str = "private.key";

/* Plugins */
const PLUGINS_DIRECTORY: &str = "plugins";
const DATA_DIRECTORY: &str = "data";

pub struct Storage;

impl Storage {
    /* Logs */
    pub fn latest_log_file() -> PathBuf {
        PathBuf::from(LOGS_DIRECTORY).join(LATEST_LOG_FILE)
    }

    /* Nodes */
    pub fn nodes_directory() -> PathBuf {
        PathBuf::from(NODES_DIRECTORY)
    }
    pub fn node_file<'a, T>(name: T) -> PathBuf where T: Into<Cow<'a, str>> {
        let name = name.into();
        Storage::nodes_directory().join(format!("{name}.toml"))
    }

    /* Groups */
    pub fn groups_directory() -> PathBuf {
        PathBuf::from(GROUPS_DIRECTORY)
    }
    pub fn group_file<'a, T>(name: T) -> PathBuf where T: Into<Cow<'a, str>> {
        let name = name.into();
        Storage::groups_directory().join(format!("{name}.toml"))
    }

    /* Auth */
    pub fn users_directory() -> PathBuf {
        PathBuf::from(USERS_DIRECTORY)
    }
    pub fn user_file<'a, T>(name: T) -> PathBuf where T: Into<Cow<'a, str>> {
        let name = name.into();
        Storage::users_directory().join(format!("{name}.toml"))
    }

    /* Configs */
    pub fn configs_directory() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn primary_config_file() -> PathBuf {
        Storage::configs_directory().join(PRIMARY_CONFIG_FILE)
    }

    /* Wasm Configs */
    pub fn wasm_plugins_config_file() -> PathBuf {
        Storage::configs_directory().join(WASM_PLUGINS_CONFIG_FILE)
    }
    pub fn wasm_engine_config_file() -> PathBuf {
        Storage::configs_directory().join(WASM_ENGINE_CONFIG_FILE)
    }

    /* Certificates */
    pub fn cert_directory() -> PathBuf {
        PathBuf::from(CERT_DIRECTORY)
    }
    pub fn cert_file() -> PathBuf {
        Storage::cert_directory().join(CERT_FILE)
    }
    pub fn cert_private_key_file() -> PathBuf {
        Storage::cert_directory().join(CERT_PRIVATE_KEY_FILE)
    }

    /* Plugins */
    pub fn plugins_directory() -> PathBuf {
        PathBuf::from(PLUGINS_DIRECTORY)
    }
    pub fn data_directory_for_plugin<'a, T>(name: T) -> PathBuf where T: Into<Cow<'a, str>> {
        PathBuf::from(DATA_DIRECTORY).join(name.into().into_owned())
    }
    pub fn config_directory_for_plugin<'a, T>(name: T) -> PathBuf where T: Into<Cow<'a, str>> {
        Storage::configs_directory().join(name.into().into_owned())
    }

    pub async fn for_each_content(path: &Path) -> Result<Vec<(PathBuf, String, String)>> {
        let mut result = Vec::new();
        let mut directory = fs::read_dir(path).await?;
        while let Some(entry) = directory.next_entry().await? {
            if entry.path().is_dir() {
                continue;
            }
            let path = entry.path();
            match (path.file_name(), path.file_stem()) {
                (Some(name), Some(stem)) => result.push((
                    path.clone(),
                    name.to_string_lossy().to_string(),
                    stem.to_string_lossy().to_string(),
                )),
                _ => {
                    warn!("Failed to read file names: {:?}", path);
                }
            }
        }
        Ok(result)
    }

    pub async fn for_each_content_toml<T: LoadFromTomlFile>(
        path: &Path,
        error_message: &str,
    ) -> Result<Vec<(PathBuf, String, String, T)>> {
        let mut result = Vec::new();
        let mut directory = fs::read_dir(path).await?;
        while let Some(entry) = directory.next_entry().await? {
            if entry.path().is_dir() {
                continue;
            }
            match T::from_file(&entry.path()).await {
                Ok(value) => {
                    let path = entry.path();
                    match (path.file_name(), path.file_stem()) {
                        (Some(name), Some(stem)) => result.push((
                            path.clone(),
                            name.to_string_lossy().to_string(),
                            stem.to_string_lossy().to_string(),
                            value,
                        )),
                        _ => {
                            warn!("Failed to read file names: {:?}", path);
                        }
                    }
                }
                Err(error) => {
                    warn!("{}@{:?}: {:?}", error_message, entry.path(), error);
                }
            }
        }
        Ok(result)
    }
}

pub trait SaveToTomlFile: Serialize {
    async fn save(&self, path: &Path, create_parent: bool) -> Result<()> {
        if create_parent {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
        }
        fs::write(path, toml::to_string(self)?).await?;
        Ok(())
    }
}

pub trait LoadFromTomlFile: DeserializeOwned {
    async fn from_file(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path).await?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
