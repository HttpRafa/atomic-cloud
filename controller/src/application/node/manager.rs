use std::collections::HashMap;

use anyhow::Result;
use common::file::for_each_content_toml;
use simplelog::{error, info, warn};
use stored::StoredNode;
use tokio::fs;

use crate::{
    application::plugin::{manager::PluginManager, BoxedNode},
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

        for (_, _, name, value) in
            for_each_content_toml::<StoredNode>(&directory, "Failed to read node from file")?
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

    pub fn get_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    pub fn has_node(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    pub fn get_node(&self, name: &str) -> Option<&Node> {
        self.nodes.get(name)
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

mod stored {
    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use getset::Getters;
    use serde::{Deserialize, Serialize};

    use crate::application::node::{Capabilities, LifecycleStatus, RemoteController};

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

    impl LoadFromTomlFile for StoredNode {}
    impl SaveToTomlFile for StoredNode {}
}
