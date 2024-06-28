use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use colored::Colorize;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::controller::{ControllerHandle, WeakControllerHandle};

use super::{
    group::WeakGroupHandle,
    node::{AllocationHandle, NodeHandle, WeakNodeHandle},
};

const EXPECTED_STARTUP_TIME: Duration = Duration::from_secs(60);
const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(15);

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
        // Check health of servers
        {
            let mut servers = self.servers.lock().unwrap();
            let dead_servers = servers.iter().filter(|server| {
                if server.health.is_dead() {
                    match server.state {
                        State::Starting => {
                            warn!("Server {} {} to establish online status within the expected startup time of {}.", server.name.blue(), "failed".red(), format!("{:.2?}", EXPECTED_STARTUP_TIME).blue());
                        }
                        _ => {
                            warn!("Server {} has not checked in for {:.2?}, indicating a potential failure.", server.name.red(), format!("{:.2?}", server.health.timeout).blue());
                        }
                    }
                    true
                } else {
                    false
                }
            }).cloned().collect::<Vec<_>>();
            for server in dead_servers {
                self.stop_server_nolock(&server, &mut servers);
            }
        }

        // Sort requests by priority and process them
        {
            let mut requests = self.requests.lock().unwrap();
            requests
                .make_contiguous()
                .sort_unstable_by_key(|req| req.priority);
            'handle_request: while let Some(request) = requests.pop_back() {
                // Collect and sort nodes by the number of allocations

                for node in &request.nodes {
                    let node = node.upgrade().unwrap();
                    // Try to allocate resources on nodes
                    if let Ok(allocation) =
                        node.allocate(&request.resources, request.deployment.clone())
                    {
                        self.start_server(&request, allocation, &node);
                        continue 'handle_request;
                    }
                }
                warn!(
                    "{} to allocate resources for server {}",
                    "Failed".red(),
                    request.name.red()
                );
            }
        }
    }

    pub fn queue_server(&self, request: StartRequest) {
        self.requests.lock().unwrap().push_back(request);
    }

    fn stop_server_nolock(&self, server: &ServerHandle, servers: &mut Vec<ServerHandle>) {
        info!("{} server {}", "Stopping".yellow(), server.name.blue());

        // Remove resources allocated by server from node
        if let Some(node) = server.node.upgrade() {
            node.deallocate(&server.allocation);
        }

        // Send start request to node
        // We do this async because the driver chould be running blocking code like network requests
        if let Some(controller) = self.controller.upgrade() {
            let server = server.clone();
            controller
                .get_runtime()
                .spawn_blocking(move || stop_thread(server));
        }

        // Remove server from group and servers list
        if let Some(group) = server.group.upgrade() {
            group.remove_server(server);
        }
        servers.retain(|handle| !Arc::ptr_eq(handle, server));

        fn stop_thread(server: ServerHandle) {
            if let Some(node) = server.node.upgrade() {
                if let Err(error) = node.get_inner().stop_server(&server) {
                    error!(
                        "{} to stop server {}: {}",
                        "Failed".red(),
                        server.name.red(),
                        error
                    );
                }
            }
        }
    }

    pub fn stop_server(&self, server: &ServerHandle) {
        self.stop_server_nolock(server, &mut self.servers.lock().unwrap());
    }

    fn start_server(
        &self,
        request: &StartRequest,
        allocation: AllocationHandle,
        node: &NodeHandle,
    ) {
        info!(
            "{} server {} on node {} listening on {}",
            "Starting".green(),
            request.name.blue(),
            node.name.blue(),
            allocation.primary_address().to_string().blue()
        );
        let server = Arc::new(Server {
            name: request.name.clone(),
            uuid: Uuid::new_v4(),
            group: request.group.clone(),
            node: Arc::downgrade(node),
            allocation,
            health: Health::new(EXPECTED_STARTUP_TIME, DEFAULT_HEALTH_CHECK_TIMEOUT),
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
            controller
                .get_runtime()
                .spawn_blocking(move || start_thread(copy, server));
        }

        fn start_thread(controller: ControllerHandle, server: ServerHandle) {
            if let Some(node) = server.node.upgrade() {
                if let Err(error) = node.get_inner().start_server(&server) {
                    error!(
                        "{} to start server {}: {}",
                        "Failed".red(),
                        server.name.red(),
                        error
                    );
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

    /* Health and State of the server */
    health: Health,
    state: State,
}

pub struct Health {
    pub next_checkin: Instant,
    pub timeout: Duration,
}

impl Health {
    pub fn new(startup_time: Duration, timeout: Duration) -> Self {
        Self {
            next_checkin: Instant::now() + startup_time,
            timeout,
        }
    }
    pub fn reset(&mut self) {
        self.next_checkin = Instant::now() + self.timeout;
    }
    pub fn is_dead(&self) -> bool {
        Instant::now() > self.next_checkin
    }
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
    pub deployment: Deployment,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Resources {
    pub memory: u32,
    pub cpu: u32,
    pub disk: u32,
    pub addresses: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum Retention {
    #[serde(rename = "delete")]
    #[default]
    Delete,
    #[serde(rename = "keep")]
    Keep,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DriverSetting {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Deployment {
    pub driver_settings: Vec<DriverSetting>,
    pub disk_retention: Retention,
}
