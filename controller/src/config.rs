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

#[derive(Deserialize, Serialize)]
pub struct NetworkConfig {
    pub bind: SocketAddr,
}

#[derive(Deserialize, Serialize)]
pub struct Timings {
    pub startup: Duration,
    pub restart: Duration,
    pub healthbeat: Duration,
    pub transfer: Duration,
    pub empty_unit: Duration,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    /* Cloud Identification */
    pub identifier: String,

    /* Network */
    pub network: NetworkConfig,

    /* Timings */
    pub timings: Timings,
}

impl Config {
    fn default() -> Self {
        Self {
            identifier: Uuid::new_v4().to_string(),
            network: NetworkConfig {
                bind: SocketAddr::new(
                    DEFAULT_BIND_ADDRESS.parse().unwrap(),
                    DEFAULT_BIND_PORT,
                ),
            },
            timings: Timings {
                startup: DEFAULT_EXPECTED_STARTUP_TIME,
                restart: DEFAULT_EXPECTED_RESTART_TIME,
                healthbeat: DEFAULT_HEALTH_CHECK_TIMEOUT,
                transfer: DEFAULT_TRANSFER_TIMEOUT,
                empty_unit: DEFAULT_EMPTY_UNIT_TIMEOUT,
            },
        }
    }

    pub fn load() -> Self {
        let path = Storage::get_primary_config_file();
        let mut config = if path.exists() {
            Self::load_from_file(&path).unwrap_or_else(|error| {
                warn!(
                    "<red>Failed</> to read configuration from file: <red>{}</>",
                    error
                );
                Self::default()
            })
        } else {
            Self::default()
        };

                // Check config values are overridden by environment variables
                if let Ok(identifier) = std::env::var("INSTANCE_IDENTIFIER") {
                    config.identifier = identifier;
                }
                if let Ok(address) = std::env::var("BIND_ADDRESS") {
                    if let Ok(address) = address.parse() {
                        config.network.bind = address;
                    } else {
                        error!("<red>Failed</> to parse BIND_ADDRESS environment variable");
                    }
                }

                config
    }
}

impl SaveToTomlFile for Config {}
impl LoadFromTomlFile for Config {}
