use std::collections::HashMap;

use anyhow::Result;
use simplelog::{error, info, warn};
use stored::StoredNode;
use tokio::fs;
use url::Url;

use crate::{
    application::{
        group::manager::GroupManager,
        plugin::{manager::PluginManager, BoxedNode},
        server::manager::ServerManager,
    },
    resource::{CreateResourceError, DeleteResourceError},
    storage::Storage,
};

use super::{Capabilities, Node};

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

            let Some(plugin) = plugins.get_plugin(value.plugin()) else {
                warn!(
                    "Plugin {} is not loaded, skipping node {}",
                    value.plugin(),
                    name
                );
                continue;
            };

            match plugin
                .init_node(&name, value.capabilities(), value.controller())
                .await
            {
                Ok(instance) => {
                    info!("Loaded node {}", name);
                    nodes.insert(name.clone(), Node::new(&name, &value, instance));
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
        groups: &GroupManager,
    ) -> Result<(), DeleteResourceError> {
        if servers.is_node_used(name) {
            return Err(DeleteResourceError::StillInUse);
        }
        if groups.is_node_used(name) {
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

    pub async fn create_node(
        &mut self,
        name: &str,
        p_name: &str,
        capabilities: &Capabilities,
        controller: &Url,
        plugins: &PluginManager,
    ) -> Result<(), CreateResourceError> {
        if self.nodes.contains_key(name) {
            return Err(CreateResourceError::AlreadyExists);
        }

        let Some(plugin) = plugins.get_plugin(p_name) else {
            return Err(CreateResourceError::RequiredPluginNotLoaded);
        };
        let node = StoredNode::new(p_name, capabilities.clone(), controller.clone());

        let instance = match plugin
            .init_node(name, node.capabilities(), node.controller())
            .await
        {
            Ok(instance) => instance,
            Err(error) => return Err(CreateResourceError::Error(error)),
        };

        let node = Node::new(name, &node, instance);
        node.save().await.map_err(CreateResourceError::Error)?;
        self.nodes.insert(name.to_string(), node);
        info!("Created node {}", name);
        Ok(())
    }

    pub fn verify_nodes(&self, names: &[String]) -> bool {
        for name in names {
            if !self.nodes.contains_key(name) {
                return true;
            }
        }
        false
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
    pub fn new(name: &str, node: &StoredNode, instance: BoxedNode) -> Self {
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
    pub fn tick(&mut self) -> Result<()> {
        for node in self.nodes.values() {
            node.tick()?;
        }
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

pub(super) mod stored {
    use getset::Getters;
    use serde::{Deserialize, Serialize};
    use url::Url;

    use crate::{
        application::node::{Capabilities, LifecycleStatus, Node},
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
        controller: Url,
    }

    impl StoredNode {
        pub fn new(plugin: &str, capabilities: Capabilities, controller: Url) -> Self {
            Self {
                plugin: plugin.to_string(),
                capabilities,
                status: LifecycleStatus::Inactive,
                controller,
            }
        }

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
