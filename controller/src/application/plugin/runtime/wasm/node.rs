use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::{spawn, sync::Mutex, task::JoinHandle};
use wasmtime::{component::ResourceAny, AsContextMut, Store};

use crate::application::plugin::GenericNode;

use super::{generated, PluginState};

pub struct PluginNode {
    bindings: Arc<generated::Plugin>,
    store: Arc<Mutex<Store<PluginState>>>,
    instance: ResourceAny,
}

impl PluginNode {
    pub fn new(
        bindings: Arc<generated::Plugin>,
        store: Arc<Mutex<Store<PluginState>>>,
        instance: ResourceAny,
    ) -> Self {
        Self {
            bindings,
            store,
            instance,
        }
    }

    fn get(
        &self,
    ) -> (
        Arc<generated::Plugin>,
        Arc<Mutex<Store<PluginState>>>,
        ResourceAny,
    ) {
        (self.bindings.clone(), self.store.clone(), self.instance)
    }
}

impl GenericNode for PluginNode {
    fn tick(&self) -> JoinHandle<Result<()>> {
        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .generic_plugin()
                .call_tick(store.lock().await.as_context_mut(), instance)
                .await
            {
                Ok(result) => result.map_err(|errors| {
                    anyhow!(errors
                        .iter()
                        .map(|error| format!("Scope: {}, Message: {}", error.scope, error.message))
                        .collect::<Vec<_>>()
                        .join("\n"))
                }),
                Err(error) => Err(error),
            }
        })
    }
}
