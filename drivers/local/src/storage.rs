/*
All the storage related functions are implemented here.
This makes it easier to change them in the future
*/

use std::path::PathBuf;

/* Configs */
//const CONFIG_DIRECTORY: &str = "/configs";

/* Data */
const DATA_DIRECTORY: &str = "/data";
//const TEMPLATES_DIRECTORY: &str = "templates";
const UNITS_DIRECTORY: &str = "units";

/* Units */
const TEMPORARY_DIRECTORY: &str = "temporary";
//const PERMANENT_DIRECTORY: &str = "permanent";

pub struct Storage;

impl Storage {
    /* Configs */
    /*pub fn get_configs_folder() -> PathBuf {
        PathBuf::from(CONFIG_DIRECTORY)
    }*/

    /* Data */
    pub fn get_data_folder() -> PathBuf {
        PathBuf::from(DATA_DIRECTORY)
    }
    pub fn get_units_folder() -> PathBuf {
        Self::get_data_folder().join(UNITS_DIRECTORY)
    }

    /* Units */
    pub fn get_temporary_folder() -> PathBuf {
        Self::get_units_folder().join(TEMPORARY_DIRECTORY)
    }
    /*pub fn get_permanent_folder() -> PathBuf {
        Self::get_units_folder().join(PERMANENT_DIRECTORY)
    }*/
}
