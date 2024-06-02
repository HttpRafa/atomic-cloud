use std::{fs, path::Path, sync::{Arc, Mutex}};
use anyhow::Result;
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use stored::StoredNode;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};
use super::driver::{Drivers, GenericDriver};

const NODES_DIRECTORY: &str = "nodes";

type NodeHandle = Arc<Mutex<Node>>;

pub enum NodeCreationResult {
    Created,
    AlreadyExists,
    Denied(String),
}

pub struct Nodes {
    nodes: Vec<NodeHandle>,
}

impl Nodes {
    pub async fn load_all(drivers: &Drivers) -> Self {
        info!("Loading nodes...");

        let node_directory = Path::new(NODES_DIRECTORY);
        if !node_directory.exists() {
            fs::create_dir_all(node_directory).unwrap_or_else(|error| {
                warn!("{} to create nodes directory: {}", "Failed".red(), &error)
            });
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

            let node = match Node::try_from(&name, &node, drivers) {
                Some(node) => node,
                None => continue,
            };

            match node.init().await {
                Ok(None) => {
                    info!("Loaded node {}", &node.name.blue());
                    nodes.push(Arc::new(Mutex::new(node)));
                }
                Ok(Some(error)) => {
                    warn!("{} to load node {} because it was denied by the driver", "Failed".red(), &node.name);
                    warn!(" -> {}", &error);
                }
                Err(error) => {
                    error!("{} to load node {}: {}", "Failed".red(), &node.name, &error);
                }
            }
        }

        info!("Loaded {}", format!("{} node(s)", nodes.len()).blue());
        Self { nodes }
    }

    pub async fn create_node(&mut self, name: &str, driver: Arc<dyn GenericDriver>, capabilities: Vec<Capability>) -> Result<NodeCreationResult> {
        if self.nodes.iter().any(|node| node.lock().unwrap().name == name) {
            return Ok(NodeCreationResult::AlreadyExists);
        }
        
        let stored_node = StoredNode { driver: driver.name().to_string(), capabilities };
        let node = Node::from(name, &stored_node, driver);

        match node.init().await {
            Ok(None) => {
                stored_node.save_to_file(&Path::new(NODES_DIRECTORY).join(format!("{}.toml", name)))?;
                self.nodes.push(Arc::new(Mutex::new(node)));
                info!("Created node {}", name.blue());
                Ok(NodeCreationResult::Created)
            }
            Ok(Some(error)) => Ok(NodeCreationResult::Denied(error)),
            Err(error) => Err(error),
        }
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
    fn from(name: &str, stored_node: &StoredNode, driver: Arc<dyn GenericDriver>) -> Self {
        Self {
            name: name.to_string(),
            capabilities: stored_node.capabilities.clone(),
            driver,
        }
    }

    fn try_from(name: &str, stored_node: &StoredNode, drivers: &Drivers) -> Option<Self> {
        drivers.find_by_name(&stored_node.driver).map(|driver| Self::from(name, stored_node, driver)).or_else(|| {
            error!(
                "{} to load node {} because there is no loaded driver with the name {}",
                "Failed".red(),
                &name.red(),
                &stored_node.driver.red()
            );
            None
        })
    }

    pub async fn init(&self) -> Result<Option<String>> {
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
    MaxServers(u32),
    #[serde(rename = "sub_node")]
    SubNode(String),
}

mod stored {
    use serde::{Deserialize, Serialize};
    use crate::config::{LoadFromTomlFile, SaveToTomlFile};
    use super::Capability;

    #[derive(Serialize, Deserialize)]
    pub struct StoredNode {
        pub driver: String,
        pub capabilities: Vec<Capability>,
    }

    impl LoadFromTomlFile for StoredNode {}
    impl SaveToTomlFile for StoredNode {}
}