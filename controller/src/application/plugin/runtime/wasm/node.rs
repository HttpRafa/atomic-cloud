use std::sync::Arc;

use anyhow::{anyhow, Result};
use common::{error::FancyError, network::HostAndPort};
use tokio::{spawn, sync::Mutex, task::JoinHandle};
use tonic::async_trait;
use wasmtime::{component::ResourceAny, AsContextMut, Store};

use crate::application::{
    node::Allocation,
    plugin::{BoxedScreen, GenericNode},
    server::{guard::Guard, manager::StartRequest, DiskRetention, Resources, Server, Spec},
};

use super::{
    ext::screen::PluginScreen,
    generated::{self, exports::plugin::system::bridge},
    PluginState,
};

pub struct PluginNode {
    dropped: bool,

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
        ResourceAny,
    ) {
        (self.bindings.clone(), self.store.clone(), self.instance)
    }
}

#[async_trait]
impl GenericNode for PluginNode {
    fn tick(&self) -> JoinHandle<Result<()>> {
        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .node()
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

    fn allocate(&self, request: &StartRequest) -> JoinHandle<Result<Vec<HostAndPort>>> {
        let proposal: bridge::ServerProposal = request.into();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .node()
                .call_allocate(store.lock().await.as_context_mut(), instance, &proposal)
                .await
            {
                Ok(result) => result
                    .map(|ports| {
                        ports
                            .into_iter()
                            .map(|port| HostAndPort::new(port.host, port.port))
                            .collect()
                    })
                    .map_err(|error| anyhow!(error)),
                Err(error) => Err(error),
            }
        })
    }

    fn free(&self, ports: &[HostAndPort]) -> JoinHandle<Result<()>> {
        let ports = ports
            .iter()
            .map(std::convert::Into::into)
            .collect::<Vec<_>>();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            bindings
                .plugin_system_bridge()
                .node()
                .call_free(store.lock().await.as_context_mut(), instance, &ports)
                .await
        })
    }

    fn start(&self, server: &Server) -> JoinHandle<Result<BoxedScreen>> {
        let server = server.into();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .node()
                .call_start(store.lock().await.as_context_mut(), instance, &server)
                .await
            {
                Ok(screen) => Ok(Box::new(PluginScreen::new(
                    bindings.clone(),
                    store.clone(),
                    screen,
                )) as BoxedScreen),
                Err(error) => Err(error),
            }
        })
    }

    fn restart(&self, server: &Server) -> JoinHandle<Result<()>> {
        let server = server.into();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            bindings
                .plugin_system_bridge()
                .node()
                .call_restart(store.lock().await.as_context_mut(), instance, &server)
                .await
        })
    }

    fn stop(&self, server: &Server, guard: Guard) -> JoinHandle<Result<()>> {
        let server = server.into();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            let mut store = store.lock().await;
            let guard = store.data_mut().new_guard(guard)?;

            bindings
                .plugin_system_bridge()
                .node()
                .call_stop(store.as_context_mut(), instance, &server, guard)
                .await
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
        self.instance
            .resource_drop_async(self.store.lock().await.as_context_mut())
            .await?;
        self.dropped = true;

        Ok(())
    }
}

impl Drop for PluginNode {
    fn drop(&mut self) {
        if !self.dropped {
            FancyError::print_fancy(
                &anyhow!("Resource was not dropped before being deallocated (memory leak)"),
                false,
            );
        }
    }
}

impl From<&HostAndPort> for bridge::Address {
    fn from(val: &HostAndPort) -> Self {
        bridge::Address {
            host: val.host.clone(),
            port: val.port,
        }
    }
}

impl From<&DiskRetention> for bridge::DiskRetention {
    fn from(val: &DiskRetention) -> Self {
        match val {
            DiskRetention::Permanent => bridge::DiskRetention::Permanent,
            DiskRetention::Temporary => bridge::DiskRetention::Temporary,
        }
    }
}

impl From<&Spec> for bridge::Spec {
    fn from(val: &Spec) -> Self {
        bridge::Spec {
            settings: val
                .settings()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            environment: val
                .environment()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            disk_retention: val.disk_retention().into(),
            image: val.image().clone(),
        }
    }
}

impl From<&Resources> for bridge::Resources {
    fn from(val: &Resources) -> Self {
        bridge::Resources {
            memory: *val.memory(),
            swap: *val.swap(),
            cpu: *val.cpu(),
            io: *val.io(),
            disk: *val.disk(),
            ports: *val.ports(),
        }
    }
}

impl From<&Allocation> for bridge::Allocation {
    fn from(val: &Allocation) -> Self {
        bridge::Allocation {
            ports: val.ports.iter().map(std::convert::Into::into).collect(),
            resources: val.resources().into(),
            spec: (&val.spec).into(),
        }
    }
}

impl From<&Server> for bridge::Server {
    fn from(val: &Server) -> Self {
        bridge::Server {
            name: val.id().name().clone(),
            uuid: val.id().uuid().to_string(),
            group: val.group().clone(),
            allocation: val.allocation().into(),
            token: val.token().clone(),
        }
    }
}

impl From<&StartRequest> for bridge::ServerProposal {
    fn from(val: &StartRequest) -> Self {
        bridge::ServerProposal {
            name: val.id().name().clone(),
            group: val.group().clone(),
            resources: val.resources().into(),
            spec: val.spec().into(),
        }
    }
}
