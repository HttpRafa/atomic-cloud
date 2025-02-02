use std::{collections::HashMap, fs};

use anyhow::Result;
use common::file::for_each_content_toml;
use simplelog::{error, info, warn};
use stored::StoredNode;

use crate::{
    application::plugin::{manager::PluginManager, WrappedNode},
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
            fs::create_dir_all(&directory)?;
        }

        for (_, _, name, value) in
            for_each_content_toml::<StoredNode>(&directory, "Failed to read node from file")?
        {
            info!("Loading node {}", name);

            let plugin = match plugins.get_plugin(value.get_plugin()) {
                Some(plugin) => plugin,
                None => {
                    warn!(
                        "Plugin {} is not loaded, skipping node {}",
                        value.get_plugin(),
                        name
                    );
                    continue;
                }
            };

            match plugin
                .init_node(&name, value.get_capabilities(), value.get_controller())
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

    pub fn has_node(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }
}

impl Node {
    pub fn new(name: &str, node: StoredNode, instance: WrappedNode) -> Self {
        Self {
            plugin: node.get_plugin().to_string(),
            instance,
            name: name.to_owned(),
            capabilities: node.get_capabilities().clone(),
            status: node.get_status().clone(),
            controller: node.get_controller().clone(),
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
    use serde::{Deserialize, Serialize};

    use crate::application::node::{Capabilities, LifecycleStatus, RemoteController};

    #[derive(Serialize, Deserialize)]
    pub struct StoredNode {
        /* Settings */
        plugin: String,
        capabilities: Capabilities,
        status: LifecycleStatus,

        /* Controller */
        controller: RemoteController,
    }

    impl StoredNode {
        pub fn get_plugin(&self) -> &str {
            &self.plugin
        }
        pub fn get_capabilities(&self) -> &Capabilities {
            &self.capabilities
        }
        pub fn get_status(&self) -> &LifecycleStatus {
            &self.status
        }
        pub fn get_controller(&self) -> &RemoteController {
            &self.controller
        }
    }

    impl LoadFromTomlFile for StoredNode {}
    impl SaveToTomlFile for StoredNode {}
}
