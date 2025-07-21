use std::path::{Path, PathBuf};

use color_eyre::eyre::{Result, eyre};
use directories::ProjectDirs;
use serde::{Serialize, de::DeserializeOwned};
use tokio::fs;

/* Profiles */
const PROFILES_DIRECTORY: &str = "profiles";

/* Known hosts */
const KNOWN_HOSTS_FILE: &str = "known_hosts.toml";

pub struct Storage;

impl Storage {
    fn directories() -> Result<ProjectDirs> {
        ProjectDirs::from("io", "atomic-cloud", "atomic-cli")
            .ok_or(eyre!("Failed to get data directory"))
    }

    /* Profiles */
    pub fn profiles_directory() -> Result<PathBuf> {
        let dirs = Self::directories()?;
        Ok(dirs.data_local_dir().join(PROFILES_DIRECTORY))
    }
    pub fn profile_file(name: &str) -> Result<PathBuf> {
        Ok(Self::profiles_directory()?.join(format!("{name}.toml")))
    }

    /* Known hosts */
    pub fn known_hosts_file() -> Result<PathBuf> {
        let dirs = Self::directories()?;
        Ok(dirs.data_local_dir().join(KNOWN_HOSTS_FILE))
    }

    pub async fn for_each_content_toml<T: LoadFromTomlFile>(
        path: &Path,
    ) -> Result<Vec<(PathBuf, String, String, T)>> {
        let mut result = Vec::new();
        let mut directory = fs::read_dir(path).await?;
        while let Some(entry) = directory.next_entry().await? {
            if entry.path().is_dir() {
                continue;
            }
            let value = T::from_file(&entry.path()).await?;
            let path = entry.path();
            if let (Some(name), Some(stem)) = (path.file_name(), path.file_stem()) {
                result.push((
                    path.clone(),
                    name.to_string_lossy().to_string(),
                    stem.to_string_lossy().to_string(),
                    value,
                ));
            }
        }
        Ok(result)
    }
}

pub trait SaveToTomlFile: Serialize {
    async fn save(&self, path: &Path, create_parent: bool) -> Result<()> {
        if create_parent && let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
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
