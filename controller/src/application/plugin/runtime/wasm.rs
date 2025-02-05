use std::sync::Arc;

use anyhow::{anyhow, Result};
use generated::exports::plugin::system::bridge;
use node::PluginNode;
use tokio::{spawn, sync::Mutex, task::JoinHandle};
use tonic::async_trait;
use wasmtime::{component::ResourceAny, AsContextMut, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiView};

use crate::application::{
    node::{Capabilities, RemoteController},
    plugin::{GenericPlugin, Information, WrappedNode},
};

pub(crate) mod config;
mod ext;
pub mod init;
mod node;

pub mod generated {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "plugin",
        path: "../protocol/wit/",
        async: true,
    });
}

pub(crate) struct PluginState {
    /* Plugin */
    name: String,

    /* Wasmtime */
    wasi: WasiCtx,
    resources: ResourceTable,
}

pub(crate) struct Plugin {
    bindings: Arc<generated::Plugin>,
    store: Arc<Mutex<Store<PluginState>>>,
    instance: ResourceAny,
}

#[async_trait]
impl GenericPlugin for Plugin {
    async fn init(&self) -> Result<Information> {
        let (bindings, store, instance) = self.get();
        let mut store = store.lock().await;
        match bindings
            .plugin_system_bridge()
            .generic_plugin()
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
        remote: &RemoteController,
    ) -> Result<WrappedNode> {
        let (bindings, store, instance) = self.get();
        match bindings
            .plugin_system_bridge()
            .generic_plugin()
            .call_init_node(
                store.clone().lock().await.as_context_mut(),
                instance,
                name,
                &capabilities.into(),
                &remote.into(),
            )
            .await?
        {
            Ok(instance) => Ok(Box::new(PluginNode::new(bindings, store, instance))),
            Err(error) => Err(anyhow!(error)),
        }
    }

    fn shutdown(&self) -> JoinHandle<Result<()>> {
        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .generic_plugin()
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

impl Plugin {
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

impl WasiView for PluginState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }
}

impl From<bridge::Information> for Information {
    fn from(val: bridge::Information) -> Self {
        Information {
            authors: val.authors,
            version: val.version,
            ready: val.ready,
        }
    }
}

impl From<&Capabilities> for bridge::Capabilities {
    fn from(val: &Capabilities) -> Self {
        bridge::Capabilities {
            memory: *val.memory(),
            max_servers: *val.max_servers(),
            child: val.child().as_ref().map(|value| value.to_string()),
        }
    }
}

impl From<&RemoteController> for bridge::RemoteController {
    fn from(val: &RemoteController) -> Self {
        bridge::RemoteController {
            address: val.address().to_string(),
        }
    }
}
