use std::path::PathBuf;

use crate::storage::Storage;

use super::{
    generated::plugin::system::{
        self,
        types::{Directory, Reference},
    },
    PluginState,
};

mod file;
mod http;
mod log;
mod platform;

impl system::types::Host for PluginState {}

impl PluginState {
    pub fn get_directory(&self, name: &str, directory: &Directory) -> Result<PathBuf, String> {
        match &directory.reference {
            Reference::Controller => Ok(PathBuf::from(".").join(&directory.path)),
            Reference::Data => {
                Ok(Storage::get_data_directory_for_plugin(name).join(&directory.path))
            }
            Reference::Configs => {
                Ok(Storage::get_config_directory_for_plugin(name).join(&directory.path))
            }
        }
    }
}
