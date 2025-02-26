use anyhow::Result;
use wasmtime::component::Resource;

use crate::application::{
    plugin::runtime::wasm::{generated::plugin::system, PluginState},
    server::guard::Guard,
};

impl system::guard::Host for PluginState {}

impl system::guard::HostGuard for PluginState {
    async fn drop(&mut self, instance: Resource<Guard>) -> Result<()> {
        self.resources.delete(instance)?;
        Ok(())
    }
}

impl PluginState {
    pub fn new_guard(&mut self, guard: Guard) -> Result<Resource<Guard>> {
        Ok(self.resources.push(guard)?)
    }
}
