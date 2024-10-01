use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
    sync::{Arc, Mutex, Weak},
};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use shared::StoredGroup;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::{
    node::{NodeHandle, Nodes, WeakNodeHandle},
    server::{
        Deployment, GroupInfo, Resources, ServerHandle, Servers, StartRequest, StartRequestHandle,
    },
    CreationResult,
};

const GROUPS_DIRECTORY: &str = "groups";

pub type GroupHandle = Arc<Group>;
pub type WeakGroupHandle = Weak<Group>;

pub struct Groups {
    groups: HashMap<String, GroupHandle>,
}

impl Groups {
    pub fn load_all(nodes: &Nodes) -> Self {
        info!("Loading groups...");

        let groups_directory = Path::new(GROUPS_DIRECTORY);
        if !groups_directory.exists() {
            if let Err(error) = fs::create_dir_all(groups_directory) {
                warn!("{} to create groups directory: {}", "Failed".red(), &error);
            }
        }

        let mut groups = Self {
            groups: HashMap::new(),
        };
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

            let name = match path.file_stem() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

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

        info!(
            "Loaded {}",
            format!("{} group(s)", groups.groups.len()).blue()
        );
        groups
    }

    pub fn get_amount(&self) -> usize {
        self.groups.len()
    }

    pub fn get_groups(&self) -> &HashMap<String, GroupHandle> {
        &self.groups
    }

    pub fn tick(&self, servers: &Servers) {
        for group in self.groups.values() {
            group.tick(servers);
        }
    }

    pub fn delete_group(&mut self, _group: &GroupHandle) -> Result<()> {
        Ok(())
    }

    pub fn create_group(
        &mut self,
        name: &str,
        node_handles: Vec<NodeHandle>,
        scaling: ScalingPolicy,
        resources: Resources,
        deployment: Deployment,
    ) -> Result<CreationResult> {
        if node_handles.is_empty() {
            return Ok(CreationResult::Denied(anyhow!("No nodes provided")));
        }

        if self.groups.contains_key(name) {
            return Ok(CreationResult::AlreadyExists);
        }

        let nodes: Vec<String> = node_handles
            .iter()
            .map(|node| node.name.clone())
            .collect();

        let stored_group = StoredGroup {
            nodes,
            scaling,
            resources,
            deployment,
        };
        let group = Group::from(name, &stored_group, node_handles.iter().map(|node| Arc::downgrade(&node)).collect());

        self.add_group(group);
        stored_group.save_to_file(&Path::new(GROUPS_DIRECTORY).join(format!("{}.toml", name)))?;
        info!("Created group {}", name.blue());
        Ok(CreationResult::Created)
    }

    pub fn find_by_name(&self, name: &str) -> Option<GroupHandle> {
        self.groups.get(name).cloned()
    }

    fn add_group(&mut self, group: GroupHandle) {
        self.groups.insert(group.name.to_string(), group);
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ScalingPolicy {
    pub minimum: u32,
    pub maximum: u32,
    pub priority: i32,
}

pub enum GroupedServer {
    Queueing(StartRequestHandle),
    Active(ServerHandle),
}

pub struct Group {
    handle: WeakGroupHandle,
    pub name: String,
    pub nodes: Vec<WeakNodeHandle>,
    pub scaling: ScalingPolicy,
    pub resources: Resources,
    pub deployment: Deployment,
    id_allocator: Mutex<IdAllocator>,
    servers: Mutex<Vec<GroupedServer>>,
}

impl Group {
    fn from(name: &str, stored_group: &StoredGroup, nodes: Vec<WeakNodeHandle>) -> GroupHandle {
        Arc::new_cyclic(|handle| Self {
            handle: handle.clone(),
            name: name.to_string(),
            nodes,
            scaling: stored_group.scaling,
            resources: stored_group.resources.clone(),
            deployment: stored_group.deployment.clone(),
            id_allocator: Mutex::new(IdAllocator::new()),
            servers: Mutex::new(Vec::new()),
        })
    }

    fn try_from(name: &str, stored_group: &StoredGroup, nodes: &Nodes) -> Option<GroupHandle> {
        let node_handles: Vec<WeakNodeHandle> = stored_group
            .nodes
            .iter()
            .filter_map(|node_name| nodes.find_by_name(node_name).map(|handle| Arc::downgrade(&handle)))
            .collect();
        if node_handles.is_empty() {
            return None;
        }
        Some(Self::from(name, stored_group, node_handles))
    }

    fn tick(&self, servers: &Servers) {
        let mut group_servers = self.servers.lock().expect("Failed to lock servers");
        let mut id_allocator = self
            .id_allocator
            .lock()
            .expect("Failed to lock id allocator");

        for requested in 0..(self.scaling.minimum as usize).saturating_sub(group_servers.len()) {
            if (group_servers.len() + requested) >= self.scaling.maximum as usize {
                break;
            }

            let server_id = id_allocator.get_id();
            let request = servers.queue_server(StartRequest {
                when: None,
                name: format!("{}-{}", self.name, server_id),
                nodes: self.nodes.clone(),
                group: Some(GroupInfo {
                    server_id,
                    group: self.handle.clone(),
                }),
                resources: self.resources.clone(),
                deployment: self.deployment.clone(),
                priority: self.scaling.priority,
            });

            // Add queueing server to group
            group_servers.push(GroupedServer::Queueing(request));
        }
    }

    pub fn set_active(&self, server: ServerHandle, request: &StartRequestHandle) {
        let mut servers = self.servers.lock().expect("Failed to lock servers");
        servers.retain(|grouped| {
            if let GroupedServer::Queueing(start_request) = grouped {
                return !Arc::ptr_eq(start_request, request);
            }
            true
        });
        servers.push(GroupedServer::Active(server));
    }

    pub fn remove_server(&self, server: &ServerHandle) {
        self.servers
            .lock()
            .expect("Failed to lock servers")
            .retain(|handle| {
                if let GroupedServer::Active(s) = handle {
                    return !Arc::ptr_eq(s, server);
                }
                true
            });
        self.id_allocator
            .lock()
            .expect("Failed to lock id allocator")
            .release_id(server.group.as_ref().unwrap().server_id);
    }

    pub fn get_free_server(&self) -> Option<ServerHandle> {
        let servers = self.servers.lock().expect("Failed to lock servers");
        for server in servers.iter() {
            if let GroupedServer::Active(server) = server {
                return Some(server.clone());
            }
        }
        None
    }
}

struct IdAllocator {
    next_id: usize,
    available_ids: BTreeSet<usize>,
    active_ids: HashSet<usize>,
}

impl IdAllocator {
    fn new() -> Self {
        Self {
            next_id: 1,
            available_ids: BTreeSet::new(),
            active_ids: HashSet::new(),
        }
    }

    fn get_id(&mut self) -> usize {
        if let Some(&id) = self.available_ids.iter().next() {
            self.available_ids.remove(&id);
            self.active_ids.insert(id);
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.active_ids.insert(id);
            id
        }
    }

    fn release_id(&mut self, id: usize) {
        if self.active_ids.remove(&id) {
            self.available_ids.insert(id);
        }
    }
}

mod shared {
    use serde::{Deserialize, Serialize};

    use crate::{
        config::{LoadFromTomlFile, SaveToTomlFile},
        controller::server::{Deployment, Resources},
    };

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
