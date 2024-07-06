use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{fs, path::Path};

pub const CONFIG_DIRECTORY: &str = "/configs";

pub trait SaveToTomlFile: Serialize {
    fn save_to_file(&self, path: &Path, create_parent: bool) -> Result<()> {
        if let Some(parent) = path.parent() {
            if create_parent {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }
}

pub trait LoadFromTomlFile: DeserializeOwned {
    fn load_from_file(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }
}
