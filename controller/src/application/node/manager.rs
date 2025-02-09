use std::collections::HashMap;

use anyhow::Result;
use simplelog::{error, info, warn};
use stored::StoredNode;
use tokio::fs;
use tonic::Status;

use crate::{
    application::{
        plugin::{manager::PluginManager, BoxedNode},
        server::manager::ServerManager,
    },
    storage::Storage,
};

use super::Node;

pub struct NodeManager {
    nodes: HashMap<String, Node>,
}

impl NodeManager {
    pub async fn init(plugins: &PluginManager) -> Result<Self> {
        info!("Loading nodes...");
        let mut nodes = HashMap::new();

        let directory = Storage::nodes_directory();
        if !directory.exists() {
            fs::create_dir_all(&directory).await?;
        }

        for (_, _, name, value) in Storage::for_each_content_toml::<StoredNode>(
            &directory,
            "Failed to read node from file",
        )
        .await?
        {
            info!("Loading node {}", name);

            let plugin = match plugins.get_plugin(value.plugin()) {
                Some(plugin) => plugin,
                None => {
                    warn!(
                        "Plugin {} is not loaded, skipping node {}",
                        value.plugin(),
                        name
                    );
                    continue;
                }
            };

            match plugin
                .init_node(&name, value.capabilities(), value.controller())
                .await
            {
                Ok(instance) => {
                    info!("Loaded node {}", name);
                    nodes.insert(name.clone(), Node::new(&name, value, instance));
                }
                Err(error) => error!("Failed to initialize node {}: {}", name, error),
            }
        }

        info!("Loaded {} node(s)", nodes.len());
        Ok(Self { nodes })
    }

    pub async fn delete_node(
        &mut self,
        name: &str,
        servers: &ServerManager,
    ) -> Result<(), DeleteResourceError> {
        if servers.is_node_used(name) {
            return Err(DeleteResourceError::StillInUse);
        }
        let node = self
            .get_node_mut(name)
            .ok_or(DeleteResourceError::NotFound)?;
        node.delete().await?;
        self.nodes.remove(name);
        info!("Deleted node {}", name);
        Ok(())
    }

    pub fn get_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    pub fn has_node(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    pub fn get_node(&self, name: &str) -> Option<&Node> {
        self.nodes.get(name)
    }
    pub fn get_node_mut(&mut self, name: &str) -> Option<&mut Node> {
        self.nodes.get_mut(name)
    }
}

impl Node {
    pub fn new(name: &str, node: StoredNode, instance: BoxedNode) -> Self {
        Self {
            plugin: node.plugin().to_string(),
            instance,
            name: name.to_owned(),
            capabilities: node.capabilities().clone(),
            status: node.status().clone(),
            controller: node.controller().clone(),
        }
    }
}

// Ticking
impl NodeManager {
    pub async fn tick(&mut self) -> Result<()> {
        for node in self.nodes.values() {
            node.tick()?;
        }
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

pub(super) mod stored {
    use getset::Getters;
    use serde::{Deserialize, Serialize};

    use crate::{
        application::node::{Capabilities, LifecycleStatus, Node, RemoteController},
        storage::{LoadFromTomlFile, SaveToTomlFile},
    };

    #[derive(Serialize, Deserialize, Getters)]
    pub struct StoredNode {
        /* Settings */
        #[getset(get = "pub")]
        plugin: String,
        #[getset(get = "pub")]
        capabilities: Capabilities,
        #[getset(get = "pub")]
        status: LifecycleStatus,

        /* Controller */
        #[getset(get = "pub")]
        controller: RemoteController,
    }

    impl StoredNode {
        pub fn from(node: &Node) -> Self {
            Self {
                plugin: node.plugin.clone(),
                capabilities: node.capabilities.clone(),
                status: node.status.clone(),
                controller: node.controller.clone(),
            }
        }
    }

    impl LoadFromTomlFile for StoredNode {}
    impl SaveToTomlFile for StoredNode {}
}

pub enum DeleteResourceError {
    StillActive,
    StillInUse,
    NotFound,
    Error(anyhow::Error),
}

impl From<DeleteResourceError> for Status {
    fn from(val: DeleteResourceError) -> Self {
        match val {
            DeleteResourceError::StillActive => {
                Status::unavailable("Resource is still set to active")
            }
            DeleteResourceError::StillInUse => Status::unavailable("Resource is still in use"),
            DeleteResourceError::NotFound => Status::not_found("Resource not found"),
            DeleteResourceError::Error(error) => Status::internal(format!("Error: {}", error)),
        }
    }
}
