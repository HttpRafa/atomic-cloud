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
    pub fn get_latest_log_file() -> PathBuf {
        PathBuf::from(LOGS_DIRECTORY).join(LATEST_LOG_FILE)
    }

    /* Nodes */
    pub fn get_nodes_directory() -> PathBuf {
        PathBuf::from(NODES_DIRECTORY)
    }
    pub fn get_node_file(name: &str) -> PathBuf {
        Storage::get_nodes_directory().join(format!("{}.toml", name))
    }

    /* Groups */
    pub fn get_groups_directory() -> PathBuf {
        PathBuf::from(GROUPS_DIRECTORY)
    }
    pub fn get_group_file(name: &str) -> PathBuf {
        Storage::get_groups_directory().join(format!("{}.toml", name))
    }

    /* Auth */
    pub fn get_users_directory() -> PathBuf {
        PathBuf::from(AUTH_DIRECTORY).join(USERS_DIRECTORY)
    }
    pub fn get_user_file(name: &str) -> PathBuf {
        Storage::get_users_directory().join(format!("{}.toml", name))
    }

    /* Configs */
    pub fn get_configs_directory() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn get_primary_config_file() -> PathBuf {
        Storage::get_configs_directory().join(PRIMARY_CONFIG_FILE)
    }

    /* Wasm Configs */
    pub fn get_wasm_plugins_config_file() -> PathBuf {
        Storage::get_configs_directory().join(WASM_PLUGINS_CONFIG_FILE)
    }
    pub fn get_wasm_engine_config_file() -> PathBuf {
        Storage::get_configs_directory().join(WASM_ENGINE_CONFIG_FILE)
    }

    /* Plugins */
    pub fn get_plugins_directory() -> PathBuf {
        PathBuf::from(PLUGINS_DIRECTORY)
    }
    pub fn get_data_directory_for_plugin(name: &str) -> PathBuf {
        PathBuf::from(DATA_DIRECTORY).join(name)
    }
    pub fn get_config_directory_for_plugin(name: &str) -> PathBuf {
        Storage::get_configs_directory().join(name)
    }
}
