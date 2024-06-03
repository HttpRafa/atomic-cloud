use std::{fs, path::Path, sync::Arc};

use anyhow::Result;
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use shared::StoredGroup;
use tokio::sync::Mutex;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::{node::{NodeHandle, Nodes}, server::ServerResources, CreationResult};

const GROUPS_DIRECTORY: &str = "groups";

pub type GroupHandle = Arc<Mutex<Group>>;

pub struct Groups {
    groups: Vec<GroupHandle>,
}

impl Groups {
    pub async fn load_all(nodes: &Nodes) -> Self {
        info!("Loading groups...");

        let groups_directory = Path::new(GROUPS_DIRECTORY);
        if !groups_directory.exists() {
            fs::create_dir_all(groups_directory).unwrap_or_else(|error| {
                warn!("{} to create groups directory: {}", "Failed".red(), &error)
            });
        }

        let mut groups = Self { groups: Vec::new() };
        let entries = match fs::read_dir(groups_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read groups directory: {}", "Failed".red(), &error);
                return groups;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read group entry: {}", "Failed".red(), &error);
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let group = match StoredGroup::load_from_file(&path) {
                Ok(group) => group,
                Err(error) => {
                    error!(
                        "{} to read group {} from file({:?}): {}",
                        "Failed".red(),
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            let group = match Group::try_from(&name, &group, &nodes).await {
                Some(group) => group,
                None => continue,
            };

            groups.add_group(group).await;
            info!("Loaded group {}", &name.blue());
        }
        
        info!("Loaded {}", format!("{} group(s)", groups.groups.len()).blue());
        groups
    }

    pub fn tick(&self) {
        // Tick server manager
        // Check if all server have send there heartbeats etc..
    }

    pub async fn create_group(&mut self, name: &str, node_handles: Vec<NodeHandle>, scaling: ScalingPolicy, resources: ServerResources) -> Result<CreationResult> {
        if node_handles.is_empty() {
            return Ok(CreationResult::Denied("No nodes provided".to_string()));
        }

        for group in &self.groups {
            if group.lock().await.name == name {
                return Ok(CreationResult::AlreadyExists);
            }
        }
        
        let mut nodes = Vec::with_capacity(node_handles.len());
        for node in &node_handles {
            nodes.push(node.lock().await.name.clone());
        }

        let stored_node = StoredGroup { nodes, scaling, resources };
        let node = Group::from(name, &stored_node, node_handles);

        self.add_group(node).await;
        stored_node.save_to_file(&Path::new(GROUPS_DIRECTORY).join(format!("{}.toml", name)))?;
        info!("Created group {}", name.blue());
        Ok(CreationResult::Created)
    }

    async fn add_group(&mut self, group: Group) {
        self.groups.push(Arc::new(Mutex::new(group)));
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ScalingPolicy {
    pub min: u32,
    pub max: u32,
    pub priority: i32,
}

pub struct Group {
    name: String,
    nodes: Vec<NodeHandle>,
    scaling: ScalingPolicy,
    resources: ServerResources,
}

impl Group {
    fn from(name: &str, stored_group: &StoredGroup, nodes: Vec<NodeHandle>) -> Self {
        Self {
            name: name.to_string(),
            nodes,
            scaling: stored_group.scaling,
            resources: stored_group.resources,
        }
    }

    async fn try_from(name: &str, stored_group: &StoredGroup, nodes: &Nodes) -> Option<Self> {
        let mut node_handles = Vec::with_capacity(stored_group.nodes.len());
        for node_name in &stored_group.nodes {
            let node = nodes.find_by_name(node_name).await?;
            node_handles.push(node);
        }
        Some(Self::from(name, stored_group, node_handles))
    }
}

mod shared {
    use serde::{Deserialize, Serialize};

    use crate::{config::{LoadFromTomlFile, SaveToTomlFile}, controller::server::ServerResources};

    use super::ScalingPolicy;

    #[derive(Serialize, Deserialize)]
    pub struct StoredGroup {
        pub nodes: Vec<String>,
        pub scaling: ScalingPolicy,
        pub resources: ServerResources,
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}