use anyhow::Result;

use crate::application::plugin::runtime::wasm::{generated::plugin::system, PluginState};

impl system::tls::Host for PluginState {
    async fn get_certificate(&mut self) -> Result<Option<String>> {
        Ok(self
            .global
            .tls
            .as_ref()
            .map(|(certificate, _)| certificate.clone()))
    }
}
