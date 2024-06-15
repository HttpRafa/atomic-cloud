use std::{fs, path::Path, sync::{Arc, Mutex, Weak}};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use shared::StoredGroup;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::{node::{Nodes, WeakNodeHandle}, server::{Deployment, Resources, ServerHandle, Servers, StartRequest}, CreationResult};

const GROUPS_DIRECTORY: &str = "groups";

type GroupHandle = Arc<Group>;
pub type WeakGroupHandle = Weak<Group>;

pub struct Groups {
    groups: Vec<GroupHandle>,
}

impl Groups {
    pub fn load_all(nodes: &Nodes) -> Self {
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

            let group = match Group::try_from(&name, &group, nodes) {
                Some(group) => group,
                None => continue,
            };

            groups.add_group(group);
            info!("Loaded group {}", &name.blue());
        }
        
        info!("Loaded {}", format!("{} group(s)", groups.groups.len()).blue());
        groups
    }

    pub fn tick(&self, servers: &Servers) {
        for group in &self.groups {
            group.tick(servers);
        }
    }

    pub fn create_group(&mut self, name: &str, node_handles: Vec<WeakNodeHandle>, scaling: ScalingPolicy, resources: Resources, deployment: Deployment) -> Result<CreationResult> {
        if node_handles.is_empty() {
            return Ok(CreationResult::Denied(anyhow!("No nodes provided")));
        }

        for group in &self.groups {
            if group.name == name {
                return Ok(CreationResult::AlreadyExists);
            }
        }
        
        let mut nodes = Vec::with_capacity(node_handles.len());
        for node in &node_handles {
            if let Some(node) = node.upgrade() {
                nodes.push(node.name.clone());
            }
        }

        let stored_node = StoredGroup { nodes, scaling, resources, deployment };
        let node = Group::from(name, &stored_node, node_handles);

        self.add_group(node);
        stored_node.save_to_file(&Path::new(GROUPS_DIRECTORY).join(format!("{}.toml", name)))?;
        info!("Created group {}", name.blue());
        Ok(CreationResult::Created)
    }

    fn add_group(&mut self, group: GroupHandle) {
        self.groups.push(group);
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ScalingPolicy {
    pub minimum: u32,
    pub maximum: u32,
    pub priority: i32,
}

pub struct Group {
    handle: WeakGroupHandle,

    pub name: String,
    pub nodes: Vec<WeakNodeHandle>,
    pub scaling: ScalingPolicy,
    pub resources: Resources,
    pub deployment: Deployment,

    /* Servers that the group has started */
    servers: Mutex<Vec<ServerHandle>>,
}

impl Group {
    fn from(name: &str, stored_group: &StoredGroup, nodes: Vec<WeakNodeHandle>) -> GroupHandle {
        Arc::new_cyclic(|handle| {
            Self {
                handle: handle.clone(),
                name: name.to_string(),
                nodes,
                scaling: stored_group.scaling,
                resources: stored_group.resources.clone(),
                deployment: stored_group.deployment.clone(),
                servers: Mutex::new(Vec::new()),
            }
        })
    }

    fn try_from(name: &str, stored_group: &StoredGroup, nodes: &Nodes) -> Option<GroupHandle> {
        let mut node_handles = Vec::with_capacity(stored_group.nodes.len());
        for node_name in &stored_group.nodes {
            let node = nodes.find_by_name(node_name)?;
            node_handles.push(node);
        }
        Some(Self::from(name, stored_group, node_handles))
    }

    fn tick(&self, server_manager: &Servers) {
        let servers = self.servers.lock().expect("Failed to lock servers");
        // Create how many servers we need to start to reach the min value
        for requested in 0..(self.scaling.minimum as usize).saturating_sub(servers.len()) {
            // Check if we have reached the max value
            if (servers.len() + requested) >= self.scaling.maximum as usize {
                break;
            }

            // We need to start a server
            server_manager.queue_server(StartRequest {
                name: format!("{}-{}", self.name, (servers.len() + requested)),
                nodes: self.nodes.clone(),
                group: self.handle.clone(),
                resources: self.resources.clone(),
                deployment: self.deployment.clone(),
                priority: self.scaling.priority,
            });
        }
    }

    pub fn add_server(&self, server: ServerHandle) {
        self.servers.lock().expect("Failed to lock servers").push(server);
    }

    pub fn remove_server(&self, server: &ServerHandle) {
        self.servers.lock().expect("Failed to lock servers").retain(|handle| !Arc::ptr_eq(handle, server));
    }
}

mod shared {
    use serde::{Deserialize, Serialize};

    use crate::{config::{LoadFromTomlFile, SaveToTomlFile}, controller::server::{Deployment, Resources}};

    use super::ScalingPolicy;

    #[derive(Serialize, Deserialize)]
    pub struct StoredGroup {
        pub nodes: Vec<String>,
        pub scaling: ScalingPolicy,
        pub resources: Resources,
        pub deployment: Deployment,
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}