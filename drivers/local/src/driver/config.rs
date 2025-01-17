use std::ops::Range;

use common::config::{LoadFromTomlFile, SaveToTomlFile};
use serde::{Deserialize, Serialize};

use crate::{error, storage::Storage, warn};

#[derive(Serialize, Deserialize)]
pub struct Config {
    /* Network */
    pub ports: Range<u16>,
}

impl Config {
    fn new_empty() -> Self {
        Self {
            ports: 27000..28000,
        }
    }

    pub fn new_filled() -> Self {
        let path = Storage::get_primary_config_file();
        if path.exists() {
            Self::load_from_file(&path).unwrap_or_else(|err| {
                warn!("Failed to read configuration from file: {}", err);
                Self::new_empty()
            })
        } else {
            let config = Self::new_empty();
            if let Err(error) = config.save_to_file(&path, false) {
                error!("Failed to save default configuration to file: {}", &error);
            }
            config
        }
    }
}

impl LoadFromTomlFile for Config {}
impl SaveToTomlFile for Config {}
