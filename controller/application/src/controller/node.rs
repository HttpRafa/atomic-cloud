use std::{fs, path::Path, sync::{Arc, Weak}};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use stored::StoredNode;
use tokio::sync::Mutex;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};
use super::{driver::{DriverHandle, DriverNodeHandle, Drivers, GenericDriver}, server::{DeploySetting, Resources}, CreationResult};

const NODES_DIRECTORY: &str = "nodes";

type NodeHandle = Arc<Mutex<Node>>;
pub type WeakNodeHandle = Weak<Mutex<Node>>;

pub struct Nodes {
    nodes: Vec<NodeHandle>,
}

impl Nodes {
    pub async fn load_all(drivers: &Drivers) -> Self {
        info!("Loading nodes...");

        let nodes_directory = Path::new(NODES_DIRECTORY);
        if !nodes_directory.exists() {
            fs::create_dir_all(nodes_directory).unwrap_or_else(|error| {
                warn!("{} to create nodes directory: {}", "Failed".red(), &error)
            });
        }

        let mut nodes = Self { nodes: Vec::new() };
        let entries = match fs::read_dir(nodes_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read nodes directory: {}", "Failed".red(), &error);
                return nodes;
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

            match nodes.add_node(node).await {
                Ok(None) => {
                    info!("Loaded node {}", &name.blue());
                }
                Ok(Some(error)) => {
                    warn!("{} to load node {} because it was denied by the driver", "Failed".red(), &name.blue());
                    warn!(" -> {}", &error);
                }
                Err(error) => {
                    error!("{} to load node {}: {}", "Failed".red(), &name.blue(), &error);
                }
            }
        }

        info!("Loaded {}", format!("{} node(s)", nodes.nodes.len()).blue());
        nodes
    }

    pub fn get_amount(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_nodes(&self) -> &Vec<NodeHandle> {
        &self.nodes
    }

    pub async fn find_by_name(&self, name: &str) -> Option<WeakNodeHandle> {
        for node in &self.nodes {
            if node.lock().await.name.eq_ignore_ascii_case(name) {
                return Some(Arc::downgrade(node));
            }
        }
        None
    }

    pub async fn create_node(&mut self, name: &str, driver: Arc<dyn GenericDriver>, capabilities: Vec<Capability>) -> Result<CreationResult> {
        for node in &self.nodes {
            if node.lock().await.name == name {
                return Ok(CreationResult::AlreadyExists);
            }
        }

        let stored_node = StoredNode { driver: driver.name().to_string(), capabilities };
        let node = Node::from(name, &stored_node, driver);

        match self.add_node(node).await {
            Ok(None) => {
                stored_node.save_to_file(&Path::new(NODES_DIRECTORY).join(format!("{}.toml", name)))?;
                info!("Created node {}", name.blue());
                Ok(CreationResult::Created)
            }
            Ok(Some(error)) => Ok(CreationResult::Denied(error)),
            Err(error) => Err(error),
        }
    }

    async fn add_node(&mut self, mut node: Node) -> Result<Option<String>> {
        match node.init().await {
            Ok(None) => {
                self.nodes.push(Arc::new(Mutex::new(node)));
                Ok(None)
            }
            Ok(Some(error)) => Ok(Some(error)),
            Err(error) => Err(error),
        }
    }
}

pub type AllocationHandle = Arc<Allocation>;
pub type WeakAllocationHandle = Weak<Allocation>;

pub struct Allocation {
    pub ports: Vec<u32>,
    pub resources: Resources,
    pub deployment: Vec<DeploySetting>,
}

pub struct Node {
    pub name: String,
    pub capabilities: Vec<Capability>,
    pub driver: DriverHandle,
    pub node: Option<DriverNodeHandle>,

    /* Allocations made on this node */
    pub allocations: Vec<WeakAllocationHandle>,
}

impl Node {
    fn from(name: &str, stored_node: &StoredNode, driver: Arc<dyn GenericDriver>) -> Self {
        Self {
            name: name.to_string(),
            capabilities: stored_node.capabilities.clone(),
            driver,
            node: None,
            allocations: Vec::new(),
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

    pub async fn init(&mut self) -> Result<Option<String>> {
        match self.driver.init_node(self).await {
            Ok(value) => match value {
                Ok(node) => {
                    self.node = Some(node);
                    Ok(None)
                },
                Err(error) => Ok(Some(error)),
            },
            Err(error) => Err(error),
        }
    }

    pub async fn allocate(&mut self, resources: Resources, deployment: Vec<DeploySetting>) -> Result<AllocationHandle> {
        for capability in &self.capabilities {
            match capability {
                Capability::LimitedMemory(value) => {
                    let used_memory: u32 = self.allocations.iter().map(|ele| ele.upgrade().unwrap().resources.memory).sum();
                    if used_memory > *value {
                        return Err(anyhow!("Node has reached the memory limit"));
                    }
                },
                Capability::MaxAllocations(value) => {
                    if self.allocations.len() + 1 > *value as usize {
                        return Err(anyhow!("Node has reached the maximum amount of allocations"));
                    }
                },
                _ => (),
            }
        }

        let ports = match self.node.as_ref().unwrap().allocate_ports(resources.ports).await? {
            Ok(ports) => ports,
            Err(error) => return Err(anyhow!("Driver failed to allocate ports: {}", error)),
        };

        if ports.len() < resources.ports as usize {
            return Err(anyhow!("Node did not allocate the required amount of ports"));
        }

        Ok(Arc::new(Allocation {
            ports,
            resources,
            deployment,
        }))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Capability {
    #[serde(rename = "limited_memory")]
    LimitedMemory(u32),
    #[serde(rename = "unlimited_memory")]
    UnlimitedMemory(bool),
    #[serde(rename = "max_allocations")]
    MaxAllocations(u32),
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