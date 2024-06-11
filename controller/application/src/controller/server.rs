use std::{collections::VecDeque, sync::{Arc, Mutex}};

use log::{error, info, warn};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::controller::{ControllerHandle, WeakControllerHandle};

use super::{group::WeakGroupHandle, node::{AllocationHandle, Node, WeakNodeHandle}};

pub type ServerHandle = Arc<Mutex<Server>>;

pub struct Servers {
    servers: Mutex<Vec<ServerHandle>>,

    /* Servers that should be started next controller tick */
    requests: Mutex<VecDeque<StartRequest>>,
}

impl Servers {
    pub fn new() -> Self {
        Self {
            servers: Mutex::new(Vec::new()),
            requests: Mutex::new(VecDeque::new()),
        }
    }

    pub fn tick(&self, controller: &WeakControllerHandle) {
        // Sort requests by priority and process them
        let mut requests = self.requests.lock().unwrap();
        requests.make_contiguous().sort_unstable_by_key(|req| req.priority);
        'handle_request: while let Some(request) = requests.pop_back() {
            // Collect and sort nodes by the number of allocations
    
            // Try to allocate resources on nodes
            for node in &request.nodes {
                let arc = node.upgrade().unwrap();
                let mut node = (node, &mut arc.lock().unwrap());
                if let Ok(allocation) = node.1.allocate(&request.resources, &request.deployment) {
                    self.start_server(&controller, &request, allocation, (node.0, &mut node.1));
                    break 'handle_request;
                }
            }
            warn!("{} to allocate resources for server {}", "Failed".red(), request.name.red());
        }
    }

    pub fn queue_server(&self, request: StartRequest) {
        self.requests.lock().unwrap().push_back(request);
    }

    pub fn stop_server(&self, raw_server: &ServerHandle) {
        let server = raw_server.lock().unwrap();
        info!("{} server {}", "Stopping".yellow(), server.name.blue());

        // Remove resources allocated by server from node
        if let Some(node) = server.node.upgrade() {
            node.lock().unwrap().deallocate(&server.allocation);
        }

        // Remove server from group and servers list
        if let Some(group) = server.group.upgrade() {
            group.lock().unwrap().remove_server(&raw_server);
        }
        self.servers.lock().unwrap().retain(|handle| !Arc::ptr_eq(handle, raw_server));
    }

    fn start_server(&self, controller: &WeakControllerHandle, request: &StartRequest, allocation: AllocationHandle, node: (&WeakNodeHandle, &mut Node)) {
        info!("{} server {} on node {} listening on {}", "Starting".green(), request.name.blue(), node.1.name.blue(), allocation.primary_address().to_string().blue());
        let server = Server {
            name: request.name.clone(),
            group: request.group.clone(),
            node: node.0.clone(),
            allocation,
            state: State::Starting,
        };

        let server = Arc::new(Mutex::new(server));
        if let Some(group) = request.group.upgrade() {
            group.lock().unwrap().add_server(server.clone());
        }
        self.servers.lock().unwrap().push(server.clone());

        // Send start request to node
        // We do this async because the driver chould be running blocking code
        if let Some(controller) = controller.upgrade() {
            let copy = controller.clone();
            controller.get_runtime().spawn_blocking(move || start_thread(copy, server));
        }

        fn start_thread(controller: ControllerHandle, server: ServerHandle) {
            let server_lock = server.lock().unwrap();
            if let Some(node) = server_lock.node.upgrade() {
                let server = server.clone();
                let node_lock = node.lock().unwrap();
                if let Err(error) = node_lock.get_inner().start_server(&server) {
                    error!("{} to start server {}: {}", "Failed".red(), server_lock.name.red(), error);
                    drop(server_lock); // IMPORTANT: Drop the lock before we call stop_server
                    drop(node_lock); // IMPORTANT: Drop the lock before we call stop_server
                    controller.get_servers().stop_server(&server);
                }
            }
        }
    }
}

pub struct Server {
    name: String,
    group: WeakGroupHandle,
    node: WeakNodeHandle,
    allocation: AllocationHandle,

    /* State that the server can have */
    state: State,
}

pub enum State {
    Starting,
    Running,
    Stopping,
}

pub struct StartRequest {
    pub name: String,
    pub group: WeakGroupHandle,
    pub nodes: Vec<WeakNodeHandle>,
    pub resources: Resources,
    pub deployment: Vec<DeploySetting>,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Resources {
    pub memory: u32,
    pub cpu: u32,
    pub disk: u32,
    pub addresses: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Retention {
    #[serde(rename = "keep")]
    Keep,
    #[serde(rename = "delete")]
    Delete,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DeploySetting {
    #[serde(rename = "image")]
    Image(String),
    #[serde(rename = "disk_retention")]
    DiskRetention(Retention),
}