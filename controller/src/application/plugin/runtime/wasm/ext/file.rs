use tokio::fs::remove_dir_all;

use crate::application::plugin::runtime::wasm::{
    generated::plugin::system::{
        self,
        types::{Directory, ErrorMessage},
    },
    PluginState,
};

impl system::file::Host for PluginState {
    async fn remove_dir_all(&mut self, directory: Directory) -> Result<(), ErrorMessage> {
        remove_dir_all(Self::get_directory(&self.name, &directory))
            .await
            .map_err(|error| format!("Failed to remove directory: {error}"))
    }
}
