use crate::application::plugin::runtime::wasm::{
    generated::plugin::system::{self, platform::Os},
    PluginState,
};

impl system::platform::Host for PluginState {
    async fn get_os(&mut self) -> Os {
        if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            Os::Unix
        }
    }
}
