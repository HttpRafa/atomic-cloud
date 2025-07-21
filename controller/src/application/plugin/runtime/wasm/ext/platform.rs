use anyhow::Result;

use crate::application::plugin::runtime::wasm::{
    PluginState,
    generated::plugin::system::{self, platform::Os},
};

impl system::platform::Host for PluginState {
    async fn get_os(&mut self) -> Result<Os> {
        if cfg!(target_os = "windows") {
            Ok(Os::Windows)
        } else {
            Ok(Os::Unix)
        }
    }
}
