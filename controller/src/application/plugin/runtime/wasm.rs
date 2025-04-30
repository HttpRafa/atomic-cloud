use std::sync::Arc;

use anyhow::{anyhow, Result};
use common::error::FancyError;
use generated::{exports::plugin::system::bridge, plugin::system::data_types};
use listener::PluginListener;
use node::PluginNode;
use tokio::{spawn, sync::Mutex, task::JoinHandle};
use tonic::async_trait;
use url::Url;
use wasmtime::{component::ResourceAny, AsContextMut, Engine, Store};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiView};

use crate::application::{
    node::Capabilities,
    plugin::{BoxedNode, Features, GenericPlugin, Information},
    Shared,
};

pub(crate) mod config;
mod epoch;
pub mod ext;
pub mod init;
mod listener;
mod node;

#[allow(clippy::all)]
pub mod generated {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "plugin",
        path: "../protocol/wit/",
        async: true,
        trappable_imports: true,
        with: {
            "plugin:system/guard/guard": crate::application::server::guard::Guard,
            "plugin:system/process/process-builder": super::ext::process::ProcessBuilder,
            "plugin:system/process/process": super::ext::process::Process,
        }
    });
}

pub(crate) struct PluginState {
    /* Global */
    shared: Arc<Shared>,

    /* Plugin */
    name: String,

    /* Wasmtime */
    wasi: WasiCtx,
    resources: ResourceTable,
}

pub(crate) struct Plugin {
    // State
    dropped: bool,

    // Features
    features: Features,

    // Listener
    listener: Option<Arc<Mutex<PluginListener>>>,

    #[allow(unused)]
    engine: Engine,
    bindings: Arc<generated::Plugin>,
    store: Arc<Mutex<Store<PluginState>>>,
    instance: ResourceAny,
}

#[async_trait]
impl GenericPlugin for Plugin {
    async fn init(&self) -> Result<Information> {
        let (bindings, store, instance, _) = self.get();
        let mut store = store.lock().await;
        match bindings
            .plugin_system_bridge()
            .plugin()
            .call_init(store.as_context_mut(), instance)
            .await
        {
            Ok(information) => Ok(information.into()),
            Err(error) => Err(error),
        }
    }

    async fn init_node(
        &self,
        name: &str,
        capabilities: &Capabilities,
        controller: &Url,
    ) -> Result<BoxedNode> {
        let (bindings, store, instance, _) = self.get();
        match bindings
            .plugin_system_bridge()
            .plugin()
            .call_init_node(
                store.clone().lock().await.as_context_mut(),
                instance,
                name,
                &capabilities.into(),
                controller.as_ref(),
            )
            .await?
        {
            Ok(instance) => Ok(Box::new(PluginNode::new(bindings, store, instance)) as BoxedNode),
            Err(error) => Err(anyhow!(error)),
        }
    }

    fn shutdown(&self) -> JoinHandle<Result<()>> {
        let (bindings, store, instance, _) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .plugin()
                .call_shutdown(store.lock().await.as_context_mut(), instance)
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

    fn tick(&self) -> JoinHandle<Result<()>> {
        let (bindings, store, instance, listener) = self.get();
        spawn(async move {
            let mut store = store.lock().await;

            // Execute events that need to be fired
            if let Some(listener) = listener {
                listener
                    .lock()
                    .await
                    .fire_events(&bindings, &mut store)
                    .await;
            }

            match bindings
                .plugin_system_bridge()
                .plugin()
                .call_tick(store.as_context_mut(), instance)
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

    async fn cleanup(&mut self) -> Result<()> {
        let mut store = self.store.lock().await;

        // Drop the listener
        if let Some(listener) = self.listener.take() {
            listener
                .lock()
                .await
                .cleanup(store.as_context_mut())
                .await?;
        }

        self.instance
            .resource_drop_async(store.as_context_mut())
            .await?;
        self.dropped = true;

        Ok(())
    }
}

impl Plugin {
    async fn init_listener(&self) -> Result<PluginListener> {
        let (bindings, store, instance, _) = self.get();
        let mut store = store.lock().await;
        match bindings
            .plugin_system_bridge()
            .plugin()
            .call_init_listener(store.as_context_mut(), instance)
            .await
        {
            Ok(instance) => Ok(PluginListener::new(instance)),
            Err(error) => Err(error),
        }
    }
}

impl Drop for Plugin {
    fn drop(&mut self) {
        if !self.dropped {
            FancyError::print_fancy(
                &anyhow!("Resource was not dropped before being deallocated (memory leak)"),
                false,
            );
        }
    }
}

impl Plugin {
    #[allow(clippy::complexity)]
    fn get(
        &self,
    ) -> (
        Arc<generated::Plugin>,
        Arc<Mutex<Store<PluginState>>>,
        ResourceAny,
        Option<Arc<Mutex<PluginListener>>>,
    ) {
        (
            self.bindings.clone(),
            self.store.clone(),
            self.instance,
            self.listener.clone(),
        )
    }
}

impl IoView for PluginState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl From<bridge::Information> for Information {
    fn from(val: bridge::Information) -> Self {
        Information {
            authors: val.authors,
            version: val.version,
            features: val.features.into(),
            ready: val.ready,
        }
    }
}

impl From<data_types::Features> for Features {
    fn from(value: data_types::Features) -> Self {
        let mut features = Features::empty();
        if value.contains(data_types::Features::NODE) {
            features.insert(Features::NODE);
        }
        if value.contains(data_types::Features::LISTENER) {
            features.insert(Features::LISTENER);
        }
        features
    }
}

impl From<&Capabilities> for bridge::Capabilities {
    fn from(val: &Capabilities) -> Self {
        bridge::Capabilities {
            memory: *val.memory(),
            max_servers: *val.servers(),
            child: val
                .child_node()
                .as_ref()
                .map(std::string::ToString::to_string),
        }
    }
}
