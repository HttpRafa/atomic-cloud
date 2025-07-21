use anyhow::{Result, anyhow};
use tokio::fs::remove_dir_all;

use crate::application::plugin::runtime::wasm::{
    PluginState,
    config::Permissions,
    generated::plugin::system::{
        self,
        types::{Directory, ErrorMessage},
    },
};

impl system::file::Host for PluginState {
    async fn remove_dir_all(&mut self, directory: Directory) -> Result<Result<(), ErrorMessage>> {
        // Check if the plugin has permissions
        if !self.permissions.contains(Permissions::ALLOW_REMOVE_DIR_ALL) {
            return Err(anyhow!(
                "Plugin tried to call remove_dir_all without the required permissions"
            ));
        }

        Ok(remove_dir_all(Self::get_directory(&self.name, &directory))
            .await
            .map_err(|error| format!("Failed to remove directory: {error}")))
    }
}
