use std::fs;
use std::path::Path;
use std::sync::Arc;
use colored::Colorize;

use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::config::{LoadFromFile, SaveToFile};
use crate::driver::Drivers;
use crate::driver::lua::LuaDriver;
use crate::node::Capabilities::UnlimitedMemory;
use crate::node::stored::StoredNode;

const NODES_DIRECTORY: &str = "nodes";
const DISABLED_DIRECTORY: &str = "disabled";
const EXAMPLE_FILE: &str = "example.toml";

pub struct Nodes {
    nodes: Vec<Arc<Node>>
}

impl Nodes {
    pub fn load_all(drivers: &Drivers) -> Nodes {
        info!("Loading nodes...");

        let node_directory = Path::new(NODES_DIRECTORY);
        // Create example node file
        if !node_directory.exists() {
            StoredNode {
                name: "example".to_string(),
                driver: "pelican".to_string(),
                capabilities: vec![Capabilities::LimitedMemory(1024), UnlimitedMemory(true), Capabilities::MaxServers(25)],
            }.save_toml(&node_directory.join(DISABLED_DIRECTORY).join(EXAMPLE_FILE)).unwrap_or_else(|error| warn!("{} to create example node: {}", "Failed".red(), error));
        }

        let mut nodes = Vec::new();
        let entries = match fs::read_dir(node_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read nodes directory: {}", "Failed".red(), &error);
                return Nodes { nodes };
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
            if path.is_dir() { continue; }

            let node = match StoredNode::load_from_file(&path) {
                Ok(node) => node,
                Err(error) => {
                    error!("{} to read node from file({:?}): {}", "Failed".red(), &path, &error);
                    continue;
                }
            };

            let node = Node::from(node, drivers);
            if node.is_none() { continue; }
            let node = node.unwrap();
            match node.init() {
                Ok(success) => {
                    if success {
                        info!("Loaded node {}", &node.name.blue());
                        nodes.push(Arc::new(node));
                    } else {
                        error!("Something went wrong while loading the node {}. Please view the logs", &node.name.blue());
                    }
                }
                Err(error) => error!("{} to load node {}: {}", "Failed".red(), &node.name, &error),
            }
        }

        info!("Loaded {}", format!("{} node(s)", nodes.len()).blue());
        Nodes { nodes }
    }
}

#[derive(Serialize)]
pub struct Node {
    name: String,
    capabilities: Vec<Capabilities>,
    #[serde(skip_serializing)]
    driver: Arc<LuaDriver>
}

impl Node {
    fn from(stored_node: StoredNode, drivers: &Drivers) -> Option<Node> {
        match drivers.find_by_name(&stored_node.driver) {
            Some(driver) => {
                Some(Node {
                    name: stored_node.name,
                    capabilities: stored_node.capabilities,
                    driver,
                })
            }
            None => {
                error!("There is no loaded driver with name {}", &stored_node.name);
                None
            }
        }
    }
    pub fn init(&self) -> Result<bool, mlua::Error> {
        self.driver.init_node(self)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Capabilities {
    #[serde(rename = "limited_memory")]
    LimitedMemory(u32),
    #[serde(rename = "unlimited_memory")]
    UnlimitedMemory(bool),
    #[serde(rename = "max_servers")]
    MaxServers(u16),
}

mod stored {
    use serde::{Deserialize, Serialize};

    use crate::config::{LoadFromFile, SaveToFile};
    use crate::node::Capabilities;

    #[derive(Serialize, Deserialize)]
    pub struct StoredNode {
        pub name: String,
        pub driver: String,
        pub capabilities: Vec<Capabilities>,
    }

    impl LoadFromFile for StoredNode {}
    impl SaveToFile for StoredNode {}
}