use std::{collections::HashMap, fs, ops::Range, time::Duration};

use anyhow::Result;
use common::file::SyncLoadFromTomlFile;
use serde::Deserialize;

use crate::storage::Storage;

const DEFAULT_CONFIG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/config.toml"));

#[derive(Deserialize, Default)]
struct Ports {
    range: Range<u16>,
    mappings: HashMap<String, Vec<u16>>,
}

#[derive(Deserialize, Default)]
struct Network {
    host: String,
    ports: Ports,
}

#[derive(Deserialize, Default)]
struct Timeouts {
    stop: Duration,
    restart: Duration,
}

#[derive(Deserialize, Default)]
pub struct Config {
    network: Network,
    timeouts: Timeouts,
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

    pub fn host(&self) -> &str {
        &self.network.host
    }

    pub fn range(&self) -> &Range<u16> {
        &self.network.ports.range
    }

    pub fn mappings(&self) -> &HashMap<String, Vec<u16>> {
        &self.network.ports.mappings
    }

    pub fn stop_timeout(&self) -> &Duration {
        &self.timeouts.stop
    }

    pub fn restart_timeout(&self) -> &Duration {
        &self.timeouts.restart
    }
}

impl SyncLoadFromTomlFile for Config {}
