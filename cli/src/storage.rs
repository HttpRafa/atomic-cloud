/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::PathBuf;

/* Cli */
const CLI_DIRECTORY: &str = "cli";

/* LOGS */
const LOGS_DIRECTORY: &str = "logs";
const LATEST_LOG_FILE: &str = "latest.log";

/* Profiles */
const PROFILES_DIRECTORY: &str = "profiles";

pub struct Storage;

impl Storage {
    /* Base */
    pub fn get_cli_folder() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory for current user")
            .join(CLI_DIRECTORY)
    }

    /* Logs */
    pub fn get_latest_log_file() -> PathBuf {
        Storage::get_cli_folder()
            .join(LOGS_DIRECTORY)
            .join(LATEST_LOG_FILE)
    }

    /* Profiles */
    pub fn get_profiles_folder() -> PathBuf {
        Storage::get_cli_folder().join(PROFILES_DIRECTORY)
    }
    pub fn get_profile_file(name: &str) -> PathBuf {
        Storage::get_profiles_folder().join(format!("{}.toml", name))
    }
}
