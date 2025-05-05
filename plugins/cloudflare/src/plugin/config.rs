use std::{
    fs,
    hash::{Hash, Hasher},
};

use anyhow::Result;
use common::file::SyncLoadFromTomlFile;
use serde::Deserialize;

use crate::storage::Storage;

const DEFAULT_CONFIG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/config.toml"));

#[derive(Deserialize, Default)]
pub struct Account {
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
    pub servers: String,
    pub name: String,
    pub ttl: u16,
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

impl Hash for Weight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.a.to_bits());
        state.write_u64(self.k.to_bits());
        state.write_u64(self.max.to_bits());
    }
}

impl PartialEq for Weight {
    fn eq(&self, other: &Self) -> bool {
        self.a.to_bits() == other.a.to_bits()
            && self.k.to_bits() == other.k.to_bits()
            && self.max.to_bits() == other.max.to_bits()
    }
}
impl Eq for Weight {}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.servers == other.servers
            && self.priority == other.priority
            && self.weight == other.weight
    }
}
impl Eq for Entry {}

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.servers.hash(state);
        self.priority.hash(state);
        self.weight.hash(state);
    }
}

impl SyncLoadFromTomlFile for Config {}
