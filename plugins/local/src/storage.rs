//! Paths used by the plugin.
//!
//! The plugin sees its data directory as `/data` and its config directory
//! as `/configs`. Host functions (`process.spawn`, `fs.remove-dir-all`)
//! take paths relative to the data directory instead, so most helpers
//! return a relative path that [`Storage::guest`] turns into a guest path.

use std::path::PathBuf;

use crate::generated::atomic::plugin_system::types::DiskRetention;

const DATA_DIRECTORY: &str = "/data";
const CONFIG_DIRECTORY: &str = "/configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

const TEMPLATES_DIRECTORY: &str = "templates";
const TEMPLATE_DATA_FILE: &str = "template.toml";

const SERVERS_DIRECTORY: &str = "servers";
const TEMPORARY_DIRECTORY: &str = "temporary";
const PERMANENT_DIRECTORY: &str = "permanent";

pub struct Storage;

impl Storage {
    /* Configs */
    pub fn primary_config_file() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY).join(PRIMARY_CONFIG_FILE)
    }

    /// Converts a path relative to the data directory into a guest path.
    pub fn guest(relative: &str) -> PathBuf {
        PathBuf::from(DATA_DIRECTORY).join(relative)
    }

    /* Templates */
    pub fn templates_directory() -> String {
        TEMPLATES_DIRECTORY.to_string()
    }
    pub fn template_directory(name: &str) -> String {
        format!("{TEMPLATES_DIRECTORY}/{name}")
    }
    pub fn template_data_file(name: &str) -> PathBuf {
        Self::guest(&Self::template_directory(name)).join(TEMPLATE_DATA_FILE)
    }

    /* Servers */
    pub fn temporary_directory() -> String {
        format!("{SERVERS_DIRECTORY}/{TEMPORARY_DIRECTORY}")
    }
    pub fn server_directory(node: &str, name: &str, retention: DiskRetention) -> String {
        let kind = match retention {
            DiskRetention::Temporary => TEMPORARY_DIRECTORY,
            DiskRetention::Permanent => PERMANENT_DIRECTORY,
        };
        format!("{SERVERS_DIRECTORY}/{kind}/{node}/{name}")
    }
}
