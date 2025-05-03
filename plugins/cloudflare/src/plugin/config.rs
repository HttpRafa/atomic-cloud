use std::fs;

use anyhow::Result;
use common::file::SyncLoadFromTomlFile;
use serde::Deserialize;

use crate::storage::Storage;

const DEFAULT_CONFIG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/config.toml"));

#[derive(Deserialize, Default)]
pub struct Account {
    pub mail: String,
    pub token: String,
}

#[derive(Deserialize, Clone)]
pub struct Weight {
    pub a: f64,
    pub k: f64,
    pub max: f64,
}

#[derive(Deserialize, Clone)]
pub struct Entry {
    pub zone: String,
    pub name: String,
    pub servers: String,
    pub priority: u16,
    pub weight: Weight,
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub rate: u16,
    pub account: Account,
    pub entries: Vec<Entry>,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let path = Storage::primary_config_file();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, DEFAULT_CONFIG)?;
        }
        Self::from_file(&path)
    }
}

impl SyncLoadFromTomlFile for Config {}
