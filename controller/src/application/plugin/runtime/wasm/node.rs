use std::sync::Arc;

use anyhow::{anyhow, Result};
use common::network::HostAndPort;
use tokio::{spawn, sync::Mutex, task::JoinHandle};
use wasmtime::{component::ResourceAny, AsContextMut, Store};

use crate::application::{
    node::Allocation,
    plugin::GenericNode,
    server::{manager::StartRequest, DiskRetention, Resources, Server, Spec},
};

use super::{
    generated::{self, exports::plugin::system::bridge},
    PluginState,
};

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
                .generic_node()
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
                .generic_node()
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
        let ports = ports.iter().map(|port| port.into()).collect::<Vec<_>>();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .generic_node()
                .call_free(store.lock().await.as_context_mut(), instance, &ports)
                .await
            {
                Ok(()) => Ok(()),
                Err(error) => Err(error),
            }
        })
    }

    fn start(&self, server: &Server) -> JoinHandle<Result<()>> {
        let server = server.into();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .generic_node()
                .call_start(
                    store.lock().await.as_context_mut(),
                    instance,
                    &server,
                )
                .await
            {
                Ok(()) => Ok(()),
                Err(error) => Err(error),
            }
        })
    }

    fn restart(&self, server: &Server) -> JoinHandle<Result<()>> {
        let server = server.into();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .generic_node()
                .call_restart(store.lock().await.as_context_mut(), instance, &server)
                .await
            {
                Ok(()) => Ok(()),
                Err(error) => Err(error),
            }
        })
    }

    fn stop(&self, server: &Server) -> JoinHandle<Result<()>> {
        let server = server.into();

        let (bindings, store, instance) = self.get();
        spawn(async move {
            match bindings
                .plugin_system_bridge()
                .generic_node()
                .call_stop(store.lock().await.as_context_mut(), instance, &server)
                .await
            {
                Ok(()) => Ok(()),
                Err(error) => Err(error),
            }
        })
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

impl From<(&String, &String)> for bridge::KeyValue {
    fn from(val: (&String, &String)) -> Self {
        bridge::KeyValue {
            key: val.0.clone(),
            value: val.1.clone(),
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
                .map(|setting| setting.into())
                .collect(),
            environment: val.environment().iter().map(|env| env.into()).collect(),
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
            ports: val.ports.iter().map(|address| address.into()).collect(),
            resources: val.resources().into(),
            spec: (&val.spec).into(),
        }
    }
}

impl From<&Server> for bridge::Server {
    fn from(val: &Server) -> Self {
        bridge::Server {
            name: val.name().clone(),
            uuid: val.uuid().to_string(),
            deployment: val.group().clone(),
            allocation: val.allocation().into(),
            token: val.token().clone()
        }
    }
}

impl From<&StartRequest> for bridge::ServerProposal {
    fn from(val: &StartRequest) -> Self {
        bridge::ServerProposal {
            name: val.name().clone(),
            deployment: val.group().clone(),
            resources: val.resources().into(),
            spec: val.spec().into(),
        }
    }
}
