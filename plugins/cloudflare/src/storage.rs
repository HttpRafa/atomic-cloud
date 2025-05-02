use std::path::PathBuf;

/* Configs */
const CONFIG_DIRECTORY: &str = "/configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

/* Data */
const DATA_DIRECTORY: &str = "/data";

pub struct Storage;

impl Storage {
    /* Configs */
    pub fn configs_directory() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn primary_config_file() -> PathBuf {
        Self::configs_directory().join(PRIMARY_CONFIG_FILE)
    }

    /* Data */
    pub fn data_directory(host: bool) -> PathBuf {
        PathBuf::from(DATA_DIRECTORY)
    }
}
