use std::fs;

use crate::application::plugin::runtime::wasm::{
    generated::plugin::system::{
        self,
        types::{Directory, ErrorMessage},
    },
    PluginState,
};

impl system::file::Host for PluginState {
    async fn remove_dir_all(&mut self, directory: Directory) -> Result<(), ErrorMessage> {
        fs::remove_dir_all(self.get_directory(&self.name, &directory)?)
            .map_err(|error| format!("Failed to remove directory: {}", error))
    }
}
