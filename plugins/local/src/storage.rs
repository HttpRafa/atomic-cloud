use std::path::PathBuf;

use common::name::TimedName;

use crate::generated::{
    exports::plugin::system::bridge::DiskRetention,
    plugin::system::types::{Directory, Reference},
};

/* Configs */
const CONFIG_DIRECTORY: &str = "/configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

/* Data */
const DATA_DIRECTORY: &str = "/data";
const SERVERS_DIRECTORY: &str = "servers";

/* Templates */
const TEMPLATES_DIRECTORY: &str = "templates";
const TEMPLATE_DATA_FILE: &str = "template.toml";

/* Servers */
const TEMPORARY_DIRECTORY: &str = "temporary";
const PERMANENT_DIRECTORY: &str = "permanent";

pub struct Storage;

impl Storage {
    /* Configs */
    pub fn configs_directory() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn primary_config_file() -> PathBuf {
        Storage::configs_directory().join(PRIMARY_CONFIG_FILE)
    }

    /* Data */
    pub fn data_directory(host: bool) -> PathBuf {
        if host {
            PathBuf::new()
        } else {
            PathBuf::from(DATA_DIRECTORY)
        }
    }
    pub fn servers_directory(host: bool) -> PathBuf {
        Self::data_directory(host).join(SERVERS_DIRECTORY)
    }

    /* Templates */
    pub fn templates_directory(host: bool) -> PathBuf {
        Self::data_directory(host).join(TEMPLATES_DIRECTORY)
    }
    pub fn template_directory(host: bool, name: &str) -> PathBuf {
        Self::data_directory(host)
            .join(TEMPLATES_DIRECTORY)
            .join(name)
    }
    pub fn get_template_data_file(host: bool, name: &str) -> PathBuf {
        Self::template_directory(host, name).join(TEMPLATE_DATA_FILE)
    }
    pub fn create_template_directory(name: &str) -> Directory {
        Directory {
            reference: Reference::Data,
            path: Self::template_directory(true, name)
                .to_string_lossy()
                .to_string(),
        }
    }

    /* Units */
    pub fn temporary_directory(host: bool) -> PathBuf {
        Self::servers_directory(host).join(TEMPORARY_DIRECTORY)
    }
    pub fn permanent_folder(host: bool) -> PathBuf {
        Self::servers_directory(host).join(PERMANENT_DIRECTORY)
    }
    pub fn create_temporary_directory() -> Directory {
        Directory {
            reference: Reference::Data,
            path: Self::temporary_directory(true)
                .to_string_lossy()
                .to_string(),
        }
    }

    pub fn server_folder(host: bool, name: &TimedName, retention: &DiskRetention) -> PathBuf {
        match retention {
            DiskRetention::Temporary => Self::temporary_directory(host).join(name.get_name()),
            DiskRetention::Permanent => Self::permanent_folder(host).join(name.get_name()),
        }
    }
    pub fn create_server_directory(name: &TimedName, retention: &DiskRetention) -> Directory {
        Directory {
            reference: Reference::Data,
            path: Self::server_folder(true, name, retention)
                .to_string_lossy()
                .to_string(),
        }
    }
}
