use std::sync::Arc;

use anyhow::anyhow;
use tokio::{spawn, sync::Mutex};
use wasmtime::{component::ResourceAny, AsContextMut, Store};

use crate::application::{
    plugin::runtime::wasm::{
        generated::{self, exports::plugin::system::bridge::ScreenType},
        PluginState,
    },
    server::screen::{GenericScreen, PullError, ScreenJoinHandle},
};

pub struct PluginScreen {
    bindings: Arc<generated::Plugin>,
    store: Arc<Mutex<Store<PluginState>>>,
    instance: ScreenType,
}

impl PluginScreen {
    pub fn new(
        bindings: Arc<generated::Plugin>,
        store: Arc<Mutex<Store<PluginState>>>,
        instance: ScreenType,
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
        Option<ResourceAny>,
    ) {
        (
            self.bindings.clone(),
            self.store.clone(),
            match self.instance {
                ScreenType::Supported(instance) => Some(instance),
                ScreenType::Unsupported => None,
            },
        )
    }
}

impl GenericScreen for PluginScreen {
    fn is_supported(&self) -> bool {
        match &self.instance {
            ScreenType::Unsupported => false,
            ScreenType::Supported(_) => true,
        }
    }

    fn pull(&self) -> ScreenJoinHandle {
        let (bindings, store, instance) = self.get();
        let Some(instance) = instance else {
            return spawn(async { Err(PullError::Unsupported) });
        };
        spawn(async move {
            match bindings
                .plugin_system_screen()
                .screen()
                .call_pull(store.lock().await.as_context_mut(), instance)
                .await
                .map_err(PullError::Error)?
            {
                Ok(result) => Ok(result),
                Err(error) => Err(PullError::Error(anyhow!(error))),
            }
        })
    }
}
