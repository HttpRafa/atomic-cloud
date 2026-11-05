use std::sync::Arc;

use anyhow::{Result, anyhow};
use common::network::HostAndPort;
use tokio::{spawn, task::JoinHandle};
use tonic::async_trait;

use crate::application::{
    node::Allocation,
    plugin::GenericNode,
    server::{
        DiskRetention, Resources, Server, Specification, guard::Guard, manager::StartRequest,
        screen::BoxedScreen,
    },
};

use super::{Runtime, generated::atomic::plugin_system::types, scoped_errors, screen::Screen};

/// A node of a wasmite plugin. It only holds the node name: the plugin
/// keeps everything else about the node in its kv store.
pub struct Node {
    name: Arc<str>,
    runtime: Arc<Runtime>,
}

impl Node {
    pub fn new(name: String, runtime: Arc<Runtime>) -> Self {
        Self {
            name: name.into(),
            runtime,
        }
    }
}

#[async_trait]
impl GenericNode for Node {
    fn tick(&self) -> JoinHandle<Result<()>> {
        let (name, runtime) = (self.name.clone(), self.runtime.clone());
        spawn(async move {
            runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_node()
                        .call_tick(store, &name)
                        .await
                })
                .await?
                .map_err(|errors| scoped_errors(&errors))
        })
    }

    fn allocate(&self, request: &StartRequest) -> JoinHandle<Result<Vec<HostAndPort>>> {
        let proposal = types::ServerProposal::from(request);
        let (name, runtime) = (self.name.clone(), self.runtime.clone());
        spawn(async move {
            let addresses = runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_node()
                        .call_allocate(store, &name, &proposal)
                        .await
                })
                .await?
                .map_err(|error| anyhow!("{error}"))?;
            Ok(addresses
                .into_iter()
                .map(|address| HostAndPort::new(address.host, address.port))
                .collect())
        })
    }

    fn free(&self, ports: &[HostAndPort]) -> JoinHandle<Result<()>> {
        let addresses: Vec<types::Address> = ports.iter().map(Into::into).collect();
        let (name, runtime) = (self.name.clone(), self.runtime.clone());
        spawn(async move {
            runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_node()
                        .call_free(store, &name, &addresses)
                        .await
                })
                .await?
                .map_err(|error| anyhow!("{error}"))
        })
    }

    fn start(&self, server: &Server) -> JoinHandle<Result<BoxedScreen>> {
        let server = types::Server::from(server);
        let (name, runtime) = (self.name.clone(), self.runtime.clone());
        spawn(async move {
            let screen = runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_node()
                        .call_start(store, &name, &server)
                        .await
                })
                .await?
                .map_err(|error| anyhow!("{error}"))?;
            Ok(Box::new(Screen::new(runtime.context().clone(), screen)) as BoxedScreen)
        })
    }

    fn restart(&self, server: &Server) -> JoinHandle<Result<()>> {
        let server = types::Server::from(server);
        let (name, runtime) = (self.name.clone(), self.runtime.clone());
        spawn(async move {
            runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_node()
                        .call_restart(store, &name, &server)
                        .await
                })
                .await?
                .map_err(|error| anyhow!("{error}"))
        })
    }

    fn stop(&self, server: &Server, guard: Guard) -> JoinHandle<Result<()>> {
        let server = types::Server::from(server);
        let (name, runtime) = (self.name.clone(), self.runtime.clone());
        spawn(async move {
            let context = runtime.context().clone();
            // Hand the guard to the host before calling the plugin, which may
            // report the stop as complete during the call itself.
            context.insert_guard(server.uuid.clone(), guard);
            let result = runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_node()
                        .call_stop(store, &name, &server)
                        .await
                })
                .await
                .and_then(|result| result.map_err(|error| anyhow!("{error}")));
            if result.is_err() {
                // The plugin will never report this stop, so do not keep the
                // controller waiting for it.
                context.release_guard(&server.uuid);
            }
            result
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

impl From<&HostAndPort> for types::Address {
    fn from(value: &HostAndPort) -> Self {
        Self {
            host: value.host.clone(),
            port: value.port,
        }
    }
}

impl From<&DiskRetention> for types::DiskRetention {
    fn from(value: &DiskRetention) -> Self {
        match value {
            DiskRetention::Permanent => Self::Permanent,
            DiskRetention::Temporary => Self::Temporary,
        }
    }
}

impl From<&Specification> for types::Specification {
    fn from(value: &Specification) -> Self {
        Self {
            settings: value
                .settings()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            environment: value
                .environment()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            disk_retention: value.disk_retention().into(),
            image: value.image().clone(),
        }
    }
}

impl From<&Resources> for types::Resources {
    fn from(value: &Resources) -> Self {
        Self {
            memory: *value.memory(),
            swap: *value.swap(),
            cpu: *value.cpu(),
            io: *value.io(),
            disk: *value.disk(),
            ports: *value.ports(),
        }
    }
}

impl From<&Allocation> for types::Allocation {
    fn from(value: &Allocation) -> Self {
        Self {
            ports: value.ports.iter().map(Into::into).collect(),
            resources: value.resources().into(),
            specification: (&value.specification).into(),
        }
    }
}

impl From<&Server> for types::Server {
    fn from(value: &Server) -> Self {
        Self {
            name: value.id().name().clone(),
            uuid: value.id().uuid().to_string(),
            group: value.group().clone(),
            allocation: value.allocation().into(),
            token: value.token().clone(),
            connected_users: *value.connected_users(),
        }
    }
}

impl From<&StartRequest> for types::ServerProposal {
    fn from(value: &StartRequest) -> Self {
        Self {
            name: value.id().name().clone(),
            group: value.group().clone(),
            resources: value.resources().into(),
            specification: value.specification().into(),
        }
    }
}
