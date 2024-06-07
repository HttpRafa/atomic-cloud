use std::{collections::VecDeque, sync::{Arc, Weak}};

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::{group::WeakGroupHandle, node::{AllocationHandle, WeakNodeHandle}};

type ServerHandle = Arc<Mutex<Server>>;
pub type WeakServerHandle = Weak<Mutex<Server>>;

pub struct Servers {
    servers: Vec<ServerHandle>,

    /* Server that should be started next controller tick */
    requests: VecDeque<StartRequest>,
}

impl Servers {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            requests: VecDeque::new(),
        }
    }

    pub async fn tick(&mut self) {
        // Tick server manager
        // Check if all server have send there heartbeats etc..
        // Start servers that are in the queue
        while let Some(request) = self.requests.pop_front() {
            // Check if nodes can handle the server
            for node in request.nodes {
                let node = node.upgrade().unwrap();
                let mut node = node.lock().await;
                if let Ok(allocation) = node.allocate(request.resources, request.deployment.clone()).await {
                    debug!("Starting server {} on node {} listening on ports {:?}", &request.name, &node.name, &allocation.ports);
                    break;
                }
            }
        }
    }

    pub fn queue_server(&mut self, request: StartRequest) {
        self.requests.push_back(request);
    }
}

pub struct Server {
    name: String,
    node: WeakNodeHandle,
    allocation: AllocationHandle,

    /* State that the server can have */
    state: State,
}

pub enum State {
    Starting,
    Running,
    Stopping,
    Stopped,
}

pub struct StartRequest {
    pub name: String,
    pub group: WeakGroupHandle,
    pub nodes: Vec<WeakNodeHandle>,
    pub resources: Resources,
    pub deployment: Vec<DeploySetting>,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct Resources {
    pub memory: u32,
    pub cpu: u32,
    pub disk: u32,
    pub ports: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DeploySetting {
    Image(String),
}