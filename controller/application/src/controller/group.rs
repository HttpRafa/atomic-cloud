use std::{fs, path::Path, sync::{Arc, Weak}};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use shared::StoredGroup;
use tokio::sync::Mutex;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::{node::{Nodes, WeakNodeHandle}, server::{DeploySetting, Resources, Servers, StartRequest, WeakServerHandle}, CreationResult};

const GROUPS_DIRECTORY: &str = "groups";

type GroupHandle = Arc<Mutex<Group>>;
pub type WeakGroupHandle = Weak<Mutex<Group>>;

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

            let group = match Group::try_from(&name, &group, nodes).await {
                Some(group) => group,
                None => continue,
            };

            groups.add_group(group).await;
            info!("Loaded group {}", &name.blue());
        }
        
        info!("Loaded {}", format!("{} group(s)", groups.groups.len()).blue());
        groups
    }

    pub async fn tick(&self, servers: &mut Servers) {
        for group in &self.groups {
            let mut group = group.lock().await;
            group.tick(servers).await;
        }
    }

    pub async fn create_group(&mut self, name: &str, node_handles: Vec<WeakNodeHandle>, scaling: ScalingPolicy, resources: Resources, deployment: Vec<DeploySetting>) -> Result<CreationResult> {
        if node_handles.is_empty() {
            return Ok(CreationResult::Denied(anyhow!("No nodes provided")));
        }

        for group in &self.groups {
            if group.lock().await.name == name {
                return Ok(CreationResult::AlreadyExists);
            }
        }
        
        let mut nodes = Vec::with_capacity(node_handles.len());
        for node in &node_handles {
            if let Some(node) = node.upgrade() {
                nodes.push(node.lock().await.name.clone());
            }
        }

        let stored_node = StoredGroup { nodes, scaling, resources, deployment };
        let node = Group::from(name, &stored_node, node_handles);

        self.add_group(node).await;
        stored_node.save_to_file(&Path::new(GROUPS_DIRECTORY).join(format!("{}.toml", name)))?;
        info!("Created group {}", name.blue());
        Ok(CreationResult::Created)
    }

    async fn add_group(&mut self, group: GroupHandle) {
        self.groups.push(group);
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ScalingPolicy {
    pub min: u32,
    pub max: u32,
    pub priority: i32,
}

pub struct Group {
    handle: WeakGroupHandle,

    name: String,
    nodes: Vec<WeakNodeHandle>,
    scaling: ScalingPolicy,
    resources: Resources,
    deployment: Vec<DeploySetting>,

    /* Servers that the group has started */
    servers: Vec<WeakServerHandle>,
}

impl Group {
    fn from(name: &str, stored_group: &StoredGroup, nodes: Vec<WeakNodeHandle>) -> GroupHandle {
        Arc::new_cyclic(|handle| {
            Mutex::new(Self {
                handle: handle.clone(),
                name: name.to_string(),
                nodes,
                scaling: stored_group.scaling,
                resources: stored_group.resources,
                deployment: stored_group.deployment.clone(),
                servers: Vec::new(),
            })
        })
    }

    async fn try_from(name: &str, stored_group: &StoredGroup, nodes: &Nodes) -> Option<GroupHandle> {
        let mut node_handles = Vec::with_capacity(stored_group.nodes.len());
        for node_name in &stored_group.nodes {
            let node = nodes.find_by_name(node_name).await?;
            node_handles.push(node);
        }
        Some(Self::from(name, stored_group, node_handles))
    }

    async fn tick(&mut self, servers: &mut Servers) {
        // Create how many servers we need to start to reach the min value
        for requested in 0..(self.scaling.min as usize).saturating_sub(self.servers.len()) {
            // Check if we have reached the max value
            if (self.servers.len() + requested) >= self.scaling.max as usize {
                break;
            }

            // We need to start a server
            servers.queue_server(StartRequest {
                name: format!("{}-{}", self.name, (self.servers.len() + requested)),
                nodes: self.nodes.clone(),
                group: self.handle.clone(),
                resources: self.resources,
                deployment: self.deployment.clone(),
                priority: self.scaling.priority,
            });
        }
    }
}

mod shared {
    use serde::{Deserialize, Serialize};

    use crate::{config::{LoadFromTomlFile, SaveToTomlFile}, controller::server::{DeploySetting, Resources}};

    use super::ScalingPolicy;

    #[derive(Serialize, Deserialize)]
    pub struct StoredGroup {
        pub nodes: Vec<String>,
        pub scaling: ScalingPolicy,
        pub resources: Resources,
        pub deployment: Vec<DeploySetting>,
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}