/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::PathBuf;

/* Logs */
const LOGS_DIRECTORY: &str = "logs";
const LATEST_LOG_FILE: &str = "latest.log";

/* Nodes */
const NODES_DIRECTORY: &str = "nodes";

/* Groups */
const GROUPS_DIRECTORY: &str = "groups";

/* Auth */
const AUTH_DIRECTORY: &str = "auth";
const USERS_DIRECTORY: &str = "users";

/* Configs */
const CONFIG_DIRECTORY: &str = "configs";
const PRIMARY_CONFIG_FILE: &str = "config.toml";

/* Wasm Configs */
const WASM_PLUGINS_CONFIG_FILE: &str = "wasm-plugins.toml";
const WASM_ENGINE_CONFIG_FILE: &str = "wasm-engine.toml";

/* Plugins */
const PLUGINS_DIRECTORY: &str = "plugins";
const DATA_DIRECTORY: &str = "data";

pub struct Storage;

impl Storage {
    /* Logs */
    pub fn latest_log_file() -> PathBuf {
        PathBuf::from(LOGS_DIRECTORY).join(LATEST_LOG_FILE)
    }

    /* Nodes */
    pub fn nodes_directory() -> PathBuf {
        PathBuf::from(NODES_DIRECTORY)
    }
    pub fn node_file(name: &str) -> PathBuf {
        Storage::nodes_directory().join(format!("{}.toml", name))
    }

    /* Groups */
    pub fn groups_directory() -> PathBuf {
        PathBuf::from(GROUPS_DIRECTORY)
    }
    pub fn group_file(name: &str) -> PathBuf {
        Storage::groups_directory().join(format!("{}.toml", name))
    }

    /* Auth */
    pub fn users_directory() -> PathBuf {
        PathBuf::from(AUTH_DIRECTORY).join(USERS_DIRECTORY)
    }
    pub fn user_file(name: &str) -> PathBuf {
        Storage::users_directory().join(format!("{}.toml", name))
    }

    /* Configs */
    pub fn configs_directory() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn primary_config_file() -> PathBuf {
        Storage::configs_directory().join(PRIMARY_CONFIG_FILE)
    }

    /* Wasm Configs */
    pub fn wasm_plugins_config_file() -> PathBuf {
        Storage::configs_directory().join(WASM_PLUGINS_CONFIG_FILE)
    }
    pub fn wasm_engine_config_file() -> PathBuf {
        Storage::configs_directory().join(WASM_ENGINE_CONFIG_FILE)
    }

    /* Plugins */
    pub fn plugins_directory() -> PathBuf {
        PathBuf::from(PLUGINS_DIRECTORY)
    }
    pub fn data_directory_for_plugin(name: &str) -> PathBuf {
        PathBuf::from(DATA_DIRECTORY).join(name)
    }
    pub fn config_directory_for_plugin(name: &str) -> PathBuf {
        Storage::configs_directory().join(name)
    }
}
