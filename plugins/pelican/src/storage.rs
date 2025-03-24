use std::path::PathBuf;

/* Configs */
const CONFIG_DIRECTORY: &str = "/configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

pub struct Storage;

impl Storage {
    /* Configs */
    pub fn configs_directory() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn primary_config_file() -> PathBuf {
        Storage::configs_directory().join(PRIMARY_CONFIG_FILE)
    }
}
