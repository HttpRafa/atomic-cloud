use std::{fs, path::Path};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub trait SyncSaveToTomlFile: Serialize {
    fn save(&self, path: &Path, create_parent: bool) -> Result<()> {
        if create_parent {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }
}

pub trait SyncLoadFromTomlFile: DeserializeOwned {
    fn from_file(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
