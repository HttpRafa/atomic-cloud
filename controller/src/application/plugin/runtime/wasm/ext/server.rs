use anyhow::Result;

use crate::application::plugin::runtime::wasm::{
    generated::{
        exports::plugin::system::bridge,
        plugin::system::{self},
    },
    PluginState,
};

impl system::server::Host for PluginState {
    async fn get_server(&mut self, uuid: String) -> Result<Option<bridge::Server>> {
        Ok(None)
    }
}
