use std::sync::Arc;

use anyhow::{Result, bail};
use tokio::{spawn, task::JoinHandle};
use tonic::async_trait;
use url::Url;

use crate::application::{
    node::Capabilities,
    plugin::{BoxedNode, Features, GenericPlugin, Information},
};

use super::{Runtime, generated::atomic::plugin_system::types, node::Node, scoped_errors};

pub struct Plugin {
    features: Features,
    runtime: Arc<Runtime>,
}

impl Plugin {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            features: Features::empty(),
            runtime: Arc::new(runtime),
        }
    }

    pub fn set_features(&mut self, features: Features) {
        self.features = features;
    }

    #[cfg(test)]
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}

#[async_trait]
impl GenericPlugin for Plugin {
    async fn init(&self) -> Result<Information> {
        let identifier = self.runtime.context().identifier.clone();
        let information = self
            .runtime
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_lifecycle()
                    .call_init(store, &identifier)
                    .await
            })
            .await?;
        match information {
            Ok(information) => Ok(Information {
                authors: information.authors,
                version: information.version,
                features: information.features.into(),
                ready: true,
            }),
            Err(error) => bail!("{error}"),
        }
    }

    async fn init_node(
        &self,
        name: &str,
        capabilities: &Capabilities,
        controller: &Url,
    ) -> Result<BoxedNode> {
        if !self.features.contains(Features::NODE) {
            bail!(
                "Plugin {} does not provide nodes",
                self.runtime.context().name
            );
        }
        let capabilities = types::Capabilities::from(capabilities);
        let result = self
            .runtime
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_init(store, name, &capabilities, controller.as_str())
                    .await
            })
            .await?;
        match result {
            Ok(()) => Ok(Box::new(Node::new(name.to_string(), self.runtime.clone()))),
            Err(error) => bail!("{error}"),
        }
    }

    fn shutdown(&self) -> JoinHandle<Result<()>> {
        let runtime = self.runtime.clone();
        spawn(async move {
            let identifier = runtime.context().identifier.clone();
            runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_lifecycle()
                        .call_shutdown(store, &identifier)
                        .await
                })
                .await?
                .map_err(|errors| scoped_errors(&errors))
        })
    }

    fn tick(&self) -> JoinHandle<Result<()>> {
        let runtime = self.runtime.clone();
        spawn(async move {
            let identifier = runtime.context().identifier.clone();
            runtime
                .call(async |bindings, store| {
                    bindings
                        .atomic_plugin_system_lifecycle()
                        .call_tick(store, &identifier)
                        .await
                })
                .await?
                .map_err(|errors| scoped_errors(&errors))
        })
    }

    async fn cleanup(&mut self) -> Result<()> {
        let context = self.runtime.context();
        context.processes.shutdown().await;
        context.release_all_guards();
        Ok(())
    }
}

impl From<&Capabilities> for types::Capabilities {
    fn from(value: &Capabilities) -> Self {
        Self {
            memory: *value.memory(),
            max_servers: *value.servers(),
            child: value.child_node().clone(),
        }
    }
}
