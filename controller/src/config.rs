use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::storage::{LoadFromTomlFile, Storage};

const DEFAULT_CONFIG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/config.toml"));

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
    empty_server: Duration,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    identifier: String,
    network: Network,
    timeouts: Timeouts,
}

impl Config {
    pub async fn parse() -> Result<Self> {
        let path = Storage::primary_config_file();
        if path.exists() {
            Self::from_file(&path).await
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&path, DEFAULT_CONFIG).await?;
            Self::from_file(&path).await
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn network_bind(&self) -> &SocketAddr {
        &self.network.bind
    }

    pub fn startup_timeout(&self) -> &Duration {
        &self.timeouts.startup
    }

    pub fn restart_timeout(&self) -> &Duration {
        &self.timeouts.restart
    }

    pub fn heartbeat_timeout(&self) -> &Duration {
        &self.timeouts.heartbeat
    }

    pub fn transfer_timeout(&self) -> &Duration {
        &self.timeouts.transfer
    }

    pub fn empty_server_timeout(&self) -> &Duration {
        &self.timeouts.empty_server
    }
}

impl LoadFromTomlFile for Config {}
