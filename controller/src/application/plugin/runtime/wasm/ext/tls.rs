use anyhow::Result;

use crate::application::plugin::runtime::wasm::{PluginState, generated::plugin::system};

impl system::tls::Host for PluginState {
    async fn get_certificate(&mut self) -> Result<Option<String>> {
        Ok(self
            .shared
            .tls
            .tls
            .as_ref()
            .map(|(certificate, _)| certificate.clone()))
    }
}
