use std::net::SocketAddr;
use std::time::Duration;

use common::config::{LoadFromTomlFile, SaveToTomlFile};
use serde::{Deserialize, Serialize};
use simplelog::{error, warn};
use uuid::Uuid;

use crate::storage::Storage;

const DEFAULT_EXPECTED_STARTUP_TIME: Duration = Duration::from_secs(130);
const DEFAULT_EXPECTED_RESTART_TIME: Duration = Duration::from_secs(120);
const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(15);
const DEFAULT_TRANSFER_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_EMPTY_UNIT_TIMEOUT: Duration = Duration::from_secs(120);

const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0";
const DEFAULT_BIND_PORT: u16 = 12892;

#[derive(Deserialize, Serialize, Default)]
pub struct NetworkConfig {
    pub bind: Option<SocketAddr>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Timings {
    pub startup: Option<Duration>,
    pub restart: Option<Duration>,
    pub healthbeat: Option<Duration>,
    pub transfer: Option<Duration>,
    pub empty_unit: Option<Duration>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    /* Cloud Identification */
    pub identifier: Option<String>,

    /* Network */
    pub network: NetworkConfig,

    /* Timings */
    pub timings: Timings,
}

impl Config {
    fn load_or_empty() -> Self {
        let path = Storage::get_primary_config_file();
        if !path.exists() {
            return Self::default();
        }
        Self::load_from_file(&path).unwrap_or_else(|error| {
            warn!(
                "<red>Failed</> to read configuration from file: <red>{}</>",
                error
            );
            Self::default()
        })
    }

    pub fn new_filled() -> Self {
        let mut config = Self::load_or_empty();

        let mut save = false;
        if config.identifier.is_none() {
            config.identifier = Some(Uuid::new_v4().to_string());
            save = true;
        }
        if config.network.bind.is_none() {
            config.network.bind = Some(SocketAddr::new(
                DEFAULT_BIND_ADDRESS.parse().unwrap(),
                DEFAULT_BIND_PORT,
            ));
            save = true;
        }
        if config.timings.startup.is_none() {
            config.timings.startup = Some(DEFAULT_EXPECTED_STARTUP_TIME);
            save = true;
        }
        if config.timings.restart.is_none() {
            config.timings.restart = Some(DEFAULT_EXPECTED_RESTART_TIME);
            save = true;
        }
        if config.timings.healthbeat.is_none() {
            config.timings.healthbeat = Some(DEFAULT_HEALTH_CHECK_TIMEOUT);
            save = true;
        }
        if config.timings.transfer.is_none() {
            config.timings.transfer = Some(DEFAULT_TRANSFER_TIMEOUT);
            save = true;
        }
        if config.timings.empty_unit.is_none() {
            config.timings.empty_unit = Some(DEFAULT_EMPTY_UNIT_TIMEOUT);
            save = true;
        }
        if save {
            if let Err(error) = config.save_to_file(&Storage::get_primary_config_file()) {
                error!(
                    "<red>Failed</> to save generated configuration to file: <red>{}</>",
                    &error
                );
            }
        }

        // Check config values are overridden by environment variables
        if let Ok(identifier) = std::env::var("INSTANCE_IDENTIFIER") {
            config.identifier = Some(identifier);
        }
        if let Ok(address) = std::env::var("BIND_ADDRESS") {
            if let Ok(address) = address.parse() {
                config.network.bind.replace(address);
            } else {
                error!("<red>Failed</> to parse BIND_ADDRESS environment variable");
            }
        }

        config
    }
}

impl SaveToTomlFile for Config {}
impl LoadFromTomlFile for Config {}
