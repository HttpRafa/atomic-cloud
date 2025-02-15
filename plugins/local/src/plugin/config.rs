use std::{fs, ops::Range};

use anyhow::Result;
use common::file::SyncLoadFromTomlFile;
use serde::Deserialize;

use crate::storage::Storage;

const DEFAULT_CONFIG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/config.toml"));

#[derive(Deserialize, Default)]
pub struct Config {
    address: String,
    ports: Range<u16>,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let path = Storage::primary_config_file();
        if path.exists() {
            Self::from_file(&path)
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, DEFAULT_CONFIG)?;
            Self::from_file(&path)
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn ports(&self) -> &Range<u16> {
        &self.ports
    }
}

impl SyncLoadFromTomlFile for Config {}
