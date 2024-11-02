/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::PathBuf;

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

/* Drivers */
const DRIVERS_DIRECTORY: &str = "drivers";
const DATA_DIRECTORY: &str = "data";

pub struct Storage;

impl Storage {

    /* Nodes */
    pub fn get_nodes_folder() -> PathBuf {
        PathBuf::from(NODES_DIRECTORY)
    }
    pub fn get_node_file(name: &str) -> PathBuf {
        Storage::get_nodes_folder().join(format!("{}.toml", name))
    }

    /* Groups */
    pub fn get_groups_folder() -> PathBuf {
        PathBuf::from(GROUPS_DIRECTORY)
    }
    pub fn get_group_file(name: &str) -> PathBuf {
        Storage::get_groups_folder().join(format!("{}.toml", name))
    }

    /* Auth */
    pub fn get_users_folder() -> PathBuf {
        PathBuf::from(AUTH_DIRECTORY).join(USERS_DIRECTORY)
    }
    pub fn get_user_file(name: &str) -> PathBuf {
        Storage::get_users_folder().join(format!("{}.toml", name))
    }

    /* Configs */
    pub fn get_configs_folder() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }
    pub fn get_primary_config_file() -> PathBuf {
        Storage::get_configs_folder().join(PRIMARY_CONFIG_FILE)
    }

    /* Drivers */
    pub fn get_drivers_folder() -> PathBuf {
        PathBuf::from(DRIVERS_DIRECTORY)
    }
    pub fn get_data_folder_for_driver(name: &str) -> PathBuf {
        Storage::get_drivers_folder().join(DATA_DIRECTORY).join(name)
    }
    pub fn get_config_folder_for_driver(name: &str) -> PathBuf {
        Storage::get_configs_folder().join(name)
    }

}