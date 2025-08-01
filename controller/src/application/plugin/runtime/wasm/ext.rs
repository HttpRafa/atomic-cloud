use std::path::PathBuf;

use crate::storage::Storage;

use super::{
    PluginState,
    generated::plugin::system::{
        self,
        types::{Directory, Reference},
    },
};

mod file;
pub mod guard;
mod http;
mod log;
mod platform;
pub mod process;
pub mod screen;
mod server;
mod tls;

impl system::types::Host for PluginState {}
impl system::data_types::Host for PluginState {}

impl PluginState {
    pub fn get_directory(name: &str, directory: &Directory) -> PathBuf {
        match &directory.reference {
            Reference::Controller => PathBuf::from(".").join(&directory.path),
            Reference::Data => Storage::data_directory_for_plugin(name).join(&directory.path),
            Reference::Configs => Storage::config_directory_for_plugin(name).join(&directory.path),
        }
    }
}
