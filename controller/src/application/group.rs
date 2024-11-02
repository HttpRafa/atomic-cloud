use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock, Weak,
    },
    time::Instant,
};

use anyhow::{anyhow, Result};
use colored::Colorize;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use shared::StoredGroup;

use crate::{config::{LoadFromTomlFile, SaveToTomlFile}, storage::Storage};

use super::{
    node::{LifecycleStatus, NodeHandle, Nodes, WeakNodeHandle},
    server::{
        Deployment, GroupInfo, Resources, ServerHandle, Servers, StartRequest, StartRequestHandle,
    },
    CreationResult, WeakControllerHandle,
};

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

        let groups_directory = Storage::get_groups_folder();
        if !groups_directory.exists() {
            if let Err(error) = fs::create_dir_all(&groups_directory) {
                warn!("{} to create groups directory: {}", "Failed".red(), &error);
            }
        }

        let mut groups = Self::new(controller);
        let entries = match fs::read_dir(&groups_directory) {
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
            group.tick(&self.controller, servers);
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<GroupHandle> {
        self.groups.get(name).cloned()
    }

    pub fn set_group_status(&mut self, group: &GroupHandle, status: LifecycleStatus) -> Result<()> {
        match status {
            LifecycleStatus::Retired => {
                self.retire_group(group);
                info!("Retired group {}", group.name.blue());
            }
            LifecycleStatus::Active => {
                self.activate_group(group);
                info!("Activated group {}", group.name.blue());
            }
        }
        *group.status.write().unwrap() = status;
        group.mark_dirty()?;
        Ok(())
    }

    fn retire_group(&mut self, group: &GroupHandle) {
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
    }

    fn activate_group(&mut self, _group: &GroupHandle) {}

    pub fn delete_group(&mut self, group: &GroupHandle) -> Result<()> {
        if *group.status.read().expect("Failed to lock status of group") != LifecycleStatus::Retired
        {
            return Err(anyhow!("Group is not retired"));
        }
        self.retire_group(group); // Make sure all servers are stopped
        group.delete_file()?;
        self.remove_group(group);

        let ref_count = Arc::strong_count(group);
        if ref_count > 1 {
            warn!(
                "Group {} still has strong references[{}] this chould indicate a memory leak!",
                group.name.blue(),
                format!("{}", ref_count).red()
            );
        }

        info!("Deleted group {}", group.name.blue());
        Ok(())
    }

    pub fn create_group(
        &mut self,
        name: &str,
        node_handles: Vec<NodeHandle>,
        constraints: StartConstraints,
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
            constraints,
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
        stored_group.save_to_file(&Storage::get_group_file(name))?;
        info!("Created group {}", name.blue());
        Ok(CreationResult::Created)
    }

    pub fn search_and_remove_node(&self, node: &NodeHandle) {
        for group in self.groups.values() {
            group
                .nodes
                .write()
                .expect("Failed to lock nodes list of group")
                .retain(|handle| {
                    if let Some(strong_node) = handle.upgrade() {
                        return !Arc::ptr_eq(&strong_node, node);
                    }
                    false
                });
            group.mark_dirty().expect("Failed to mark group as dirty");
        }
    }

    fn add_group(&mut self, group: GroupHandle) {
        self.groups.insert(group.name.to_string(), group);
    }

    fn remove_group(&mut self, group: &GroupHandle) {
        self.groups.remove(&group.name);
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct StartConstraints {
    pub minimum: u32,
    pub maximum: u32,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ScalingPolicy {
    pub enabled: bool,
    pub max_players: u32,
    pub start_threshold: f32,
    pub stop_empty_servers: bool,
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
    pub nodes: RwLock<Vec<WeakNodeHandle>>,
    pub constraints: StartConstraints,
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
            nodes: RwLock::new(nodes),
            constraints: stored_group.constraints,
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

    fn tick(&self, controller: &WeakControllerHandle, servers: &Servers) {
        if *self.status.read().unwrap() == LifecycleStatus::Retired {
            // Do not tick this group because it is retired
            return;
        }

        let mut group_servers = self.servers.write().expect("Failed to lock servers");
        let mut id_allocator = self
            .id_allocator
            .write()
            .expect("Failed to lock id allocator");
        let mut target_server_count = self.constraints.minimum;

        // Apply scaling policy
        if self.scaling.enabled {
            for server in group_servers.iter() {
                if let GroupedServer::Active(server) = server {
                    let player_ratio =
                        server.get_player_count() as f32 / self.scaling.max_players as f32;
                    if player_ratio >= self.scaling.start_threshold {
                        target_server_count += 1; // Server has reached the threshold, start a new one
                    }
                }
            }

            if self.scaling.stop_empty_servers && group_servers.len() as u32 > target_server_count {
                let mut amount_to_stop = group_servers.len() as u32 - target_server_count;

                // We have more servers than needed
                // Check if a server is empty and stop it after the configured timeout
                if let Some(controller) = controller.upgrade() {
                    for server in group_servers.iter() {
                        if let GroupedServer::Active(server) = server {
                            let mut stop_flag =
                                server.flags.stop.write().expect("Failed to lock stop flag");
                            if server.get_player_count() == 0 {
                                if let Some(stop_time) = stop_flag.as_ref() {
                                    if &Instant::now() > stop_time && amount_to_stop > 0 {
                                        debug!("Server {} is empty and reached the timeout, stopping...", server.name.blue());
                                        controller.get_servers().checked_stop_server(server);
                                        amount_to_stop -= 1;
                                    }
                                } else {
                                    debug!(
                                        "Server {} is empty, starting stop timer...",
                                        server.name.blue()
                                    );
                                    stop_flag.replace(
                                        Instant::now()
                                            + controller
                                                .configuration
                                                .timings
                                                .empty_server
                                                .unwrap(),
                                    );
                                }
                            } else if stop_flag.is_some() {
                                debug!(
                                    "Server {} is no longer empty, clearing stop timer...",
                                    server.name.blue()
                                );
                                stop_flag.take();
                            }
                        }
                    }
                }
            }
        }

        // Check if we need to start more servers
        for requested in 0..(target_server_count as usize).saturating_sub(group_servers.len()) {
            if (group_servers.len() + requested) >= target_server_count as usize {
                break;
            }

            let server_id = id_allocator.get_id();
            let request = servers.queue_server(StartRequest {
                canceled: AtomicBool::new(false),
                when: None,
                name: format!("{}-{}", self.name, server_id),
                nodes: self.nodes.read().unwrap().clone(),
                group: Some(GroupInfo {
                    server_id,
                    group: self.handle.clone(),
                }),
                resources: self.resources.clone(),
                deployment: self.deployment.clone(),
                priority: self.constraints.priority,
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

    pub fn mark_dirty(&self) -> Result<()> {
        self.save_to_file()
    }

    fn delete_file(&self) -> Result<()> {
        let file_path = Storage::get_group_file(&self.name);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        let stored_group = StoredGroup {
            status: self.status.read().unwrap().clone(),
            nodes: self
                .nodes
                .read()
                .unwrap()
                .iter()
                .map(|node| node.upgrade().unwrap().name.clone())
                .collect(),
            constraints: self.constraints,
            scaling: self.scaling,
            resources: self.resources.clone(),
            deployment: self.deployment.clone(),
        };
        stored_group.save_to_file(&Storage::get_group_file(&self.name))
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

    use super::{ScalingPolicy, StartConstraints};

    #[derive(Serialize, Deserialize)]
    pub struct StoredGroup {
        /* Settings */
        pub status: LifecycleStatus,

        /* Where? */
        pub nodes: Vec<String>,
        pub constraints: StartConstraints,
        pub scaling: ScalingPolicy,

        /* How? */
        pub resources: Resources,
        pub deployment: Deployment,
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}
