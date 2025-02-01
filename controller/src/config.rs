use std::{fs, net::SocketAddr, str::FromStr, time::Duration};

use anyhow::Result;
use common::config::{LoadFromTomlFile, SaveToTomlFile};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::Storage;

const DEFAULT_CONFIG: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/config.toml"));

#[derive(Deserialize, Serialize)]
struct Network {
    bind: SocketAddr,
}

#[derive(Deserialize, Serialize)]
struct Timeouts {
    startup: Duration,
    restart: Duration,
    heartbeat: Duration,
    transfer: Duration,
    empty_instance: Duration,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    identifier: String,
    network: Network,
    timeouts: Timeouts,
}

impl Config {
    pub fn parse() -> Result<Self> {
        let path = Storage::get_primary_config_file();
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

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_network_bind(&self) -> &SocketAddr {
        &self.network.bind
    }

    pub fn get_startup_timeout(&self) -> &Duration {
        &self.timeouts.startup
    }

    pub fn get_restart_timeout(&self) -> &Duration {
        &self.timeouts.restart
    }

    pub fn get_heartbeat_timeout(&self) -> &Duration {
        &self.timeouts.heartbeat
    }

    pub fn get_transfer_timeout(&self) -> &Duration {
        &self.timeouts.transfer
    }

    pub fn get_empty_instance_timeout(&self) -> &Duration {
        &self.timeouts.empty_instance
    }
}

impl LoadFromTomlFile for Config {}