use std::{mem::replace, sync::Arc};

use anyhow::{anyhow, Result};
use simplelog::error;
use tokio::{spawn, sync::Mutex};
use tonic::async_trait;
use wasmtime::{component::ResourceAny, AsContextMut, Store};

use crate::application::{
    plugin::runtime::wasm::{
        generated::{self, exports::plugin::system::bridge::ScreenType},
        PluginState,
    },
    server::screen::{GenericScreen, PullError, ScreenJoinHandle},
};

pub struct PluginScreen {
    dropped: bool,

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
            dropped: false,
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

#[async_trait]
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
                .generic_screen()
                .call_pull(store.lock().await.as_context_mut(), instance)
                .await
                .map_err(PullError::Error)?
            {
                Ok(result) => Ok(result),
                Err(error) => Err(PullError::Error(anyhow!(error))),
            }
        })
    }

    async fn drop_resources(&mut self) -> Result<()> {
        if let ScreenType::Supported(instance) =
            replace(&mut self.instance, ScreenType::Unsupported)
        {
            instance
                .resource_drop_async(self.store.lock().await.as_context_mut())
                .await?;
        }
        self.dropped = true;

        Ok(())
    }
}

impl Drop for PluginScreen {
    fn drop(&mut self) {
        if !self.dropped {
            error!("Resource was not dropped before being deallocated (memory leak)");
        }
    }
}
