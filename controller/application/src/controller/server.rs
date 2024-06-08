use std::{collections::VecDeque, sync::Arc};

use log::{error, info, warn};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::{group::WeakGroupHandle, node::{AllocationHandle, Node, WeakNodeHandle}, ControllerHandle};

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

    pub async fn tick(&self, controller: &ControllerHandle) {
        // Sort requests by priority and process them
        let mut requests = self.requests.lock().await;
        requests.make_contiguous().sort_unstable_by_key(|req| req.priority);
        'handle_request: while let Some(request) = requests.pop_back() {
            // Collect and sort nodes by the number of allocations
            let mut nodes = Vec::with_capacity(request.nodes.len());
            for node in &request.nodes {
                let temp = node.upgrade().unwrap();
                nodes.push((node, temp.lock_owned().await));
            }
            nodes.sort_unstable_by_key(|node| node.1.allocations.len());
    
            // Try to allocate resources on nodes
            for mut node in nodes {
                if let Ok(allocation) = node.1.allocate(&request.resources, &request.deployment).await {
                    self.start_server(&controller, &request, allocation, (node.0, &mut node.1)).await;
                    break 'handle_request;
                }
            }
            warn!("{} to allocate resources for server {}", "Failed".red(), request.name.red());
        }
    }

    pub async fn queue_server(&self, request: StartRequest) {
        self.requests.lock().await.push_back(request);
    }

    pub async fn stop_server(&self, raw_server: &ServerHandle) {
        let server = raw_server.lock().await;
        info!("Stopping server {}", server.name.blue());

        // Remove resources allocated by server from node
        if let Some(node) = server.node.upgrade() {
            node.lock().await.deallocate(&server.allocation).await;
        }

        // Remove server from group and servers list
        if let Some(group) = server.group.upgrade() {
            group.lock().await.remove_server(&raw_server);
        }
        self.servers.lock().await.retain(|handle| !Arc::ptr_eq(handle, raw_server));
    }

    async fn start_server(&self, controller: &ControllerHandle, request: &StartRequest, allocation: AllocationHandle, node: (&WeakNodeHandle, &mut Node)) {
        info!("Starting server {} on node {} listening on {}", request.name.blue(), node.1.name.blue(), allocation.primary_address().to_string().blue());
        let server = Server {
            name: request.name.clone(),
            group: request.group.clone(),
            node: node.0.clone(),
            allocation,
            state: State::Starting,
        };

        let server = Arc::new(Mutex::new(server));
        if let Some(group) = request.group.upgrade() {
            group.lock().await.add_server(server.clone());
        }
        self.servers.lock().await.push(server.clone());

        // Send start request to node
        // We do this async because the driver chould be running blocking code
        let controller = controller.clone();
        tokio::spawn(async move {
            let server_lock = server.lock().await;
            if let Some(node) = server_lock.node.upgrade() {
                let server = server.clone();
                let node_lock = node.lock().await;
                if let Err(error) = node_lock.get_inner().start_server(&server).await {
                    error!("Failed to start server {}: {}", server_lock.name.red(), error);
                    drop(server_lock); // IMPORTANT: Drop the lock before we call stop_server
                    drop(node_lock); // IMPORTANT: Drop the lock before we call stop_server
                    controller.upgrade().unwrap().servers.stop_server(&server).await;
                }
            }
        });
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
    Keep,
    Delete,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DeploySetting {
    Image(String),
    DiskRetention(Retention),
}