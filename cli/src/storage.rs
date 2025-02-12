/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use simplelog::warn;
use tokio::fs;

/* Cli */
const CLI_DIRECTORY: &str = "cli";

/* LOGS */
const LOGS_DIRECTORY: &str = "logs";
const LATEST_LOG_FILE: &str = "latest.log";

/* Profiles */
const PROFILES_DIRECTORY: &str = "profiles";

pub struct Storage;

impl Storage {
    /* Base */
    pub fn cli_folder() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory for current user")
            .join(CLI_DIRECTORY)
    }

    /* Logs */
    pub fn latest_log_file() -> PathBuf {
        Storage::cli_folder()
            .join(LOGS_DIRECTORY)
            .join(LATEST_LOG_FILE)
    }

    /* Profiles */
    pub fn profiles_folder() -> PathBuf {
        Storage::cli_folder().join(PROFILES_DIRECTORY)
    }
    pub fn profile_file(name: &str) -> PathBuf {
        Storage::profiles_folder().join(format!("{name}.toml"))
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
