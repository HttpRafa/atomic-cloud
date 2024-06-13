use std::{collections::VecDeque, sync::{Arc, Mutex}};

use log::{error, info, warn};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::controller::{ControllerHandle, WeakControllerHandle};

use super::{group::WeakGroupHandle, node::{AllocationHandle, NodeHandle, WeakNodeHandle}};

pub type ServerHandle = Arc<Server>;

pub struct Servers {
    controller: WeakControllerHandle,

    /* Servers started by this atomic cloud instance */
    servers: Mutex<Vec<ServerHandle>>,

    /* Servers that should be started next controller tick */
    requests: Mutex<VecDeque<StartRequest>>,
}

impl Servers {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            servers: Mutex::new(Vec::new()),
            requests: Mutex::new(VecDeque::new()),
        }
    }

    pub fn tick(&self) {
        // Sort requests by priority and process them
        let mut requests = self.requests.lock().unwrap();
        requests.make_contiguous().sort_unstable_by_key(|req| req.priority);
        'handle_request: while let Some(request) = requests.pop_back() {
            // Collect and sort nodes by the number of allocations
    
            for node in &request.nodes {
                let node = node.upgrade().unwrap();
                // Try to allocate resources on nodes
                if let Ok(allocation) = node.allocate(&request.resources, &request.deployment) {
                    self.start_server(&request, allocation, &node);
                    break 'handle_request;
                }
            }
            warn!("{} to allocate resources for server {}", "Failed".red(), request.name.red());
        }
    }

    pub fn queue_server(&self, request: StartRequest) {
        self.requests.lock().unwrap().push_back(request);
    }

    pub fn stop_server(&self, server: &ServerHandle) {
        info!("{} server {}", "Stopping".yellow(), server.name.blue());

        // Remove resources allocated by server from node
        if let Some(node) = server.node.upgrade() {
            node.deallocate(&server.allocation);
        }

        // Send start request to node
        // We do this async because the driver chould be running blocking code like network requests
        if let Some(controller) = self.controller.upgrade() {
            let server = server.clone();
            controller.get_runtime().spawn_blocking(move || stop_thread(server));
        }

        // Remove server from group and servers list
        if let Some(group) = server.group.upgrade() {
            group.remove_server(&server);
        }
        self.servers.lock().unwrap().retain(|handle| !Arc::ptr_eq(handle, server));

        fn stop_thread(server: ServerHandle) {
            if let Some(node) = server.node.upgrade() {
                if let Err(error) = node.get_inner().stop_server(&server) {
                    error!("{} to stop server {}: {}", "Failed".red(), server.name.red(), error);
                }
            }
        }
    }

    fn start_server(&self, request: &StartRequest, allocation: AllocationHandle, node: &NodeHandle) {
        info!("{} server {} on node {} listening on {}", "Starting".green(), request.name.blue(), node.name.blue(), allocation.primary_address().to_string().blue());
        let server = Arc::new(Server {
            name: request.name.clone(),
            uuid: Uuid::new_v4(),
            group: request.group.clone(),
            node: Arc::downgrade(&node),
            allocation,
            state: State::Starting,
        });

        if let Some(group) = request.group.upgrade() {
            group.add_server(server.clone());
        }
        self.servers.lock().unwrap().push(server.clone());

        // Send start request to node
        // We do this async because the driver chould be running blocking code like network requests
        if let Some(controller) = self.controller.upgrade() {
            let copy = controller.clone();
            controller.get_runtime().spawn_blocking(move || start_thread(copy, server));
        }

        fn start_thread(controller: ControllerHandle, server: ServerHandle) {
            if let Some(node) = server.node.upgrade() {
                if let Err(error) = node.get_inner().start_server(&server) {
                    error!("{} to start server {}: {}", "Failed".red(), server.name.red(), error);
                    controller.get_servers().stop_server(&server);
                }
            }
        }
    }
}

pub struct Server {
    pub name: String,
    pub uuid: Uuid,
    pub group: WeakGroupHandle,
    pub node: WeakNodeHandle,
    pub allocation: AllocationHandle,

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