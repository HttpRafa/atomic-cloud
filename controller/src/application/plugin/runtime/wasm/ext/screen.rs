use std::{mem::replace, sync::Arc};

use anyhow::{anyhow, Result};
use common::error::FancyError;
use tokio::{spawn, sync::Mutex};
use tonic::async_trait;
use wasmtime::{component::ResourceAny, AsContextMut, Store};

use crate::application::{
    plugin::runtime::wasm::{generated::{self, exports::plugin::system::screen::ScreenType}, PluginState}, server::screen::{GenericScreen, ScreenError, ScreenPullJoinHandle, ScreenWriteJoinHandle}
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

    fn pull(&self) -> ScreenPullJoinHandle {
        let (bindings, store, instance) = self.get();
        let Some(instance) = instance else {
            return spawn(async { Err(ScreenError::Unsupported) });
        };
        spawn(async move {
            match bindings
                .plugin_system_screen()
                .screen()
                .call_pull(store.lock().await.as_context_mut(), instance)
                .await
                .map_err(ScreenError::Error)?
            {
                Ok(result) => Ok(result),
                Err(error) => Err(ScreenError::Error(anyhow!(error))),
            }
        })
    }

    fn write(&self, data: &[u8]) -> ScreenWriteJoinHandle {
        let (bindings, store, instance) = self.get();
        let Some(instance) = instance else {
            return spawn(async { Err(ScreenError::Unsupported) });
        };
        let data = data.to_vec();
        spawn(async move {
            match bindings
                .plugin_system_screen()
                .screen()
                .call_write(store.lock().await.as_context_mut(), instance, &data)
                .await
                .map_err(ScreenError::Error)?
            {
                Ok(result) => Ok(result),
                Err(error) => Err(ScreenError::Error(anyhow!(error))),
            }
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
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
            FancyError::print_fancy(
                &anyhow!("Resource was not dropped before being deallocated (memory leak)"),
                false,
            );
        }
    }
}
