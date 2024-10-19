use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock, Weak,
    },
};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use shared::StoredGroup;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::{
    node::{LifecycleStatus, NodeHandle, Nodes, WeakNodeHandle},
    server::{
        Deployment, GroupInfo, Resources, ServerHandle, Servers, StartRequest, StartRequestHandle,
    },
    CreationResult, WeakControllerHandle,
};

const GROUPS_DIRECTORY: &str = "groups";

pub type GroupHandle = Arc<Group>;
pub type WeakGroupHandle = Weak<Group>;

pub struct Groups {
    controller: WeakControllerHandle,

    groups: HashMap<String, GroupHandle>,
}

impl Groups {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            groups: HashMap::new(),
        }
    }

    pub fn load_all(controller: WeakControllerHandle, nodes: &Nodes) -> Self {
        info!("Loading groups...");

        let groups_directory = Path::new(GROUPS_DIRECTORY);
        if !groups_directory.exists() {
            if let Err(error) = fs::create_dir_all(groups_directory) {
                warn!("{} to create groups directory: {}", "Failed".red(), &error);
            }
        }

        let mut groups = Groups::new(controller);
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

    pub fn find_by_name(&self, name: &str) -> Option<GroupHandle> {
        self.groups.get(name).cloned()
    }

    pub fn retire_group(&mut self, group: &GroupHandle) -> Result<()> {
        *group.status.write().unwrap() = LifecycleStatus::Retired;
        let controller = self
            .controller
            .upgrade()
            .expect("The controller is dead while still running code that requires it");
        {
            let server_manager = controller.get_servers();
            let mut servers = group.servers.write().unwrap();
            for server in servers.iter() {
                if let GroupedServer::Active(server) = server {
                    server_manager.checked_stop_server(server);
                } else if let GroupedServer::Queueing(request) = server {
                    request.canceled.store(true, Ordering::Relaxed);
                }
            }
            servers.clear();
        }
        group.save_to_file()?;
        Ok(())
    }

    pub fn delete_group(&mut self, group: &GroupHandle) -> Result<()> {
        if *group.status.read().unwrap() != LifecycleStatus::Retired {
            return Err(anyhow!("Group is not retired"));
        }
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

        let nodes: Vec<String> = node_handles.iter().map(|node| node.name.clone()).collect();

        let stored_group = StoredGroup {
            status: LifecycleStatus::Retired,
            nodes,
            scaling,
            resources,
            deployment,
        };
        let group = Group::from(
            name,
            &stored_group,
            node_handles.iter().map(Arc::downgrade).collect(),
        );

        self.add_group(group);
        stored_group.save_to_file(&Path::new(GROUPS_DIRECTORY).join(format!("{}.toml", name)))?;
        info!("Created group {}", name.blue());
        Ok(CreationResult::Created)
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

    /* Settings */
    pub name: String,
    pub status: RwLock<LifecycleStatus>,

    /* Where? */
    pub nodes: Vec<WeakNodeHandle>,
    pub scaling: ScalingPolicy,

    /* How? */
    pub resources: Resources,
    pub deployment: Deployment,

    /* What do i need to know? */
    id_allocator: RwLock<IdAllocator>,
    servers: RwLock<Vec<GroupedServer>>,
}

impl Group {
    fn from(name: &str, stored_group: &StoredGroup, nodes: Vec<WeakNodeHandle>) -> GroupHandle {
        Arc::new_cyclic(|handle| Self {
            handle: handle.clone(),
            name: name.to_string(),
            status: RwLock::new(stored_group.status.clone()),
            nodes,
            scaling: stored_group.scaling,
            resources: stored_group.resources.clone(),
            deployment: stored_group.deployment.clone(),
            id_allocator: RwLock::new(IdAllocator::new()),
            servers: RwLock::new(Vec::new()),
        })
    }

    fn try_from(name: &str, stored_group: &StoredGroup, nodes: &Nodes) -> Option<GroupHandle> {
        let node_handles: Vec<WeakNodeHandle> = stored_group
            .nodes
            .iter()
            .filter_map(|node_name| {
                nodes
                    .find_by_name(node_name)
                    .map(|handle| Arc::downgrade(&handle))
            })
            .collect();
        if node_handles.is_empty() {
            return None;
        }
        Some(Self::from(name, stored_group, node_handles))
    }

    fn tick(&self, servers: &Servers) {
        if *self.status.read().unwrap() == LifecycleStatus::Retired {
            // Do not tick this group because it is retired
            return;
        }

        let mut group_servers = self.servers.write().expect("Failed to lock servers");
        let mut id_allocator = self
            .id_allocator
            .write()
            .expect("Failed to lock id allocator");

        for requested in 0..(self.scaling.minimum as usize).saturating_sub(group_servers.len()) {
            if (group_servers.len() + requested) >= self.scaling.maximum as usize {
                break;
            }

            let server_id = id_allocator.get_id();
            let request = servers.queue_server(StartRequest {
                canceled: AtomicBool::new(false),
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

    pub fn set_server_active(&self, server: ServerHandle, request: &StartRequestHandle) {
        let mut servers = self.servers.write().expect("Failed to lock servers");
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
            .write()
            .expect("Failed to lock servers")
            .retain(|handle| {
                if let GroupedServer::Active(s) = handle {
                    return !Arc::ptr_eq(s, server);
                }
                true
            });
        self.id_allocator
            .write()
            .expect("Failed to lock id allocator")
            .release_id(server.group.as_ref().unwrap().server_id);
    }

    pub fn get_free_server(&self) -> Option<ServerHandle> {
        let servers = self.servers.read().expect("Failed to lock servers");
        for server in servers.iter() {
            if let GroupedServer::Active(server) = server {
                return Some(server.clone());
            }
        }
        None
    }

    fn save_to_file(&self) -> Result<()> {
        let stored_group = StoredGroup {
            status: self.status.read().unwrap().clone(),
            nodes: self.nodes.iter().map(|node| node.upgrade().unwrap().name.clone()).collect(),
            scaling: self.scaling,
            resources: self.resources.clone(),
            deployment: self.deployment.clone(),
        };
        stored_group.save_to_file(&Path::new(GROUPS_DIRECTORY).join(format!("{}.toml", self.name)))
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
        application::{
            node::LifecycleStatus,
            server::{Deployment, Resources},
        },
        config::{LoadFromTomlFile, SaveToTomlFile},
    };

    use super::ScalingPolicy;

    #[derive(Serialize, Deserialize)]
    pub struct StoredGroup {
        /* Settings */
        pub status: LifecycleStatus,

        /* Where? */
        pub nodes: Vec<String>,
        pub scaling: ScalingPolicy,

        /* How? */
        pub resources: Resources,
        pub deployment: Deployment,
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}
