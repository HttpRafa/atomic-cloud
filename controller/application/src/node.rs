use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::config::{LoadFromTomlFile, SaveToTomlFile};
use crate::driver::{GenericDriver, Drivers};
use crate::node::Capability::UnlimitedMemory;
use crate::node::stored::StoredNode;

const NODES_DIRECTORY: &str = "nodes";
const EXAMPLES_DIRECTORY: &str = "examples";
const EXAMPLE_FILE: &str = "example.toml";

pub struct Nodes {
    nodes: Vec<Arc<Node>>,
}

impl Nodes {
    pub async fn load_all(drivers: &Drivers) -> Self {
        info!("Loading nodes...");

        let node_directory = Path::new(NODES_DIRECTORY);
        // Create example node file if the directory doesn't exist
        if !node_directory.exists() {
            StoredNode {
                driver: "testing".to_string(),
                capabilities: vec![Capability::LimitedMemory(1024), UnlimitedMemory(true), Capability::MaxServers(25)],
            }.save_toml(&node_directory.join(EXAMPLES_DIRECTORY).join(EXAMPLE_FILE)).unwrap_or_else(|error| warn!("{} to create example node: {}", "Failed".red(), error));
        }

        let mut nodes = Vec::new();
        let entries = match fs::read_dir(node_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read nodes directory: {}", "Failed".red(), &error);
                return Self { nodes };
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read node entry: {}", "Failed".red(), &error);
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let node = match StoredNode::load_from_file(&path) {
                Ok(node) => node,
                Err(error) => {
                    error!(
                        "{} to read node {} from file({:?}): {}",
                        "Failed".red(),
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            let node = match Node::from(&name, node, drivers) {
                Some(node) => node,
                None => continue,
            };

            match node.init().await {
                Ok(success) => {
                    if success {
                        info!("Loaded node {}", &node.name.blue());
                        nodes.push(Arc::new(node));
                    } else {
                        error!("Something went wrong while loading the node {}. Please view the logs", &node.name.blue());
                    }
                }
                Err(error) => {
                    error!("{} to load node {}: {}", "Failed".red(), &node.name, &error);
                }
            }
        }

        info!("Loaded {}", format!("{} node(s)", nodes.len()).blue());
        Self { nodes }
    }
}

#[derive(Serialize)]
pub struct Node {
    pub name: String,
    pub capabilities: Vec<Capability>,
    #[serde(skip_serializing)]
    driver: Arc<dyn GenericDriver>,
}

impl Node {
    fn from(name: &str, stored_node: StoredNode, drivers: &Drivers) -> Option<Self> {
        drivers.find_by_name(&stored_node.driver).map(|driver| Self {
            name: name.to_string(),
            capabilities: stored_node.capabilities,
            driver,
        }).or_else(|| {
            error!(
                "{} to load node {} because there is no loaded driver with the name {}",
                "Failed".red(),
                &name.red(),
                &stored_node.driver.red()
            );
            None
        })
    }

    pub async fn init(&self) -> Result<bool> {
        self.driver.init_node(self).await
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Capability {
    #[serde(rename = "limited_memory")]
    LimitedMemory(u32),
    #[serde(rename = "unlimited_memory")]
    UnlimitedMemory(bool),
    #[serde(rename = "max_servers")]
    MaxServers(u16),
}

mod stored {
    use serde::{Deserialize, Serialize};

    use crate::config::{LoadFromTomlFile, SaveToTomlFile};
    use crate::node::Capability;

    #[derive(Serialize, Deserialize)]
    pub struct StoredNode {
        pub driver: String,
        pub capabilities: Vec<Capability>,
    }

    impl LoadFromTomlFile for StoredNode {}
    impl SaveToTomlFile for StoredNode {}
}