use std::{fs, time::Duration};

use anyhow::Result;
use common::file::SyncLoadFromTomlFile;
use serde::Deserialize;
use url::Url;

use crate::storage::Storage;

const DEFAULT_CONFIG: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/configs/config.toml"));

#[derive(Deserialize)]
struct Network {
    url: Url,
}

#[derive(Deserialize, Default)]
struct Application {
    token: String,
}

#[derive(Deserialize, Default)]
struct Client {
    username: String,
    token: String,
}

#[derive(Deserialize, Default)]
struct Timeouts {
    stop: Duration,
    restart: Duration,
}

#[derive(Deserialize, Default)]
pub struct Config {
    network: Network,
    application: Application,
    client: Client,
    timeouts: Timeouts,
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

    pub fn url(&self) -> &Url {
        &self.network.url
    }

    pub fn token(&self) -> &str {
        &self.application.token
    }

    pub fn username(&self) -> &str {
        &self.client.username
    }

    pub fn client_token(&self) -> &str {
        &self.client.token
    }

    pub fn stop_timeout(&self) -> &Duration {
        &self.timeouts.stop
    }

    pub fn restart_timeout(&self) -> &Duration {
        &self.timeouts.restart
    }
}

impl Default for Network {
    fn default() -> Self {
        Self {
            url: Url::parse("https://localhost:8080").unwrap(),
        }
    }
}

impl SyncLoadFromTomlFile for Config {}
