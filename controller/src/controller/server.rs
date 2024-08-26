use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Weak,
    },
    time::{Duration, Instant},
};

use colored::Colorize;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::controller::{ControllerHandle, WeakControllerHandle};

use super::{
    auth::AuthServerHandle,
    group::WeakGroupHandle,
    node::{AllocationHandle, NodeHandle, WeakNodeHandle},
};

pub type ServerHandle = Arc<Server>;
pub type WeakServerHandle = Weak<Server>;
pub type StartRequestHandle = Arc<StartRequest>;

pub struct Servers {
    controller: WeakControllerHandle,

    /* Servers started by this atomic cloud instance */
    servers: Mutex<Vec<ServerHandle>>,

    /* Servers that should be started/stopped next controller tick */
    start_requests: Mutex<VecDeque<StartRequestHandle>>,
    stop_requests: Mutex<VecDeque<StopRequest>>,
}

impl Servers {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            servers: Mutex::new(Vec::new()),
            start_requests: Mutex::new(VecDeque::new()),
            stop_requests: Mutex::new(VecDeque::new()),
        }
    }

    pub fn tick(&self) {
        // Get Controller handle
        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        // Check health of servers
        {
            let dead_servers = self.servers.lock().unwrap().iter().filter(|server| {
                let health = server.health.lock().unwrap();
                if health.is_dead() {
                    match *server.state.lock().unwrap() {
                        State::Starting | State::Restarting => {
                            warn!("Server {} {} to establish online status within the expected startup time of {}.", server.name.blue(), "failed".red(), format!("{:.2?}", controller.configuration.timings.restart.unwrap()).blue());
                        }
                        _ => {
                            warn!("Server {} has not checked in for {}, indicating a potential failure.", server.name.blue(), format!("{:.2?}", health.timeout).blue());
                        }
                    }
                    true
                } else {
                    false
                }
            }).cloned().collect::<Vec<_>>();
            for server in dead_servers {
                self.restart_server(&server);
            }
        }

        // Stop all servers that have to be stopped
        {
            let mut requests = self.stop_requests.lock().unwrap();
            requests.retain(|request| {
                if let Some(when) = request.when {
                    if when > Instant::now() {
                        return true;
                    }
                }

                self.stop_server_nolock(request, &mut self.servers.lock().unwrap());
                false
            });
        }

        // Sort requests by priority and process them
        {
            let mut requests = self.start_requests.lock().unwrap();
            {
                let contiguous = requests.make_contiguous();
                contiguous.sort_unstable_by_key(|req| req.priority);
                contiguous.reverse();
            }
            requests.retain(|request| {
                if let Some(when) = request.when {
                    if when > Instant::now() {
                        return true;
                    }
                }

                // Collect and sort nodes by the number of allocations
                for node in &request.nodes {
                    let node = node.upgrade().unwrap();
                    // Try to allocate resources on nodes
                    if let Ok(allocation) =
                        node.allocate(&request.resources, request.deployment.clone())
                    {
                        // Start server on the node
                        self.start_server(request, allocation, &node);
                        return false;
                    }
                }
                warn!(
                    "{} to allocate resources for server {}",
                    "Failed".red(),
                    request.name.red()
                );
                true
            });
        }
    }

    pub fn queue_server(&self, request: StartRequest) -> StartRequestHandle {
        let arc = Arc::new(request);
        self.start_requests.lock().unwrap().push_back(arc.clone());
        arc
    }

    pub fn stop_all(&self) {
        let mut servers = self.servers.lock().unwrap();
        while let Some(server) = servers.pop() {
            self.stop_server_nolock(&StopRequest { when: None, server }, &mut servers);
        }
    }

    fn stop_server_nolock(&self, request: &StopRequest, servers: &mut Vec<ServerHandle>) {
        let server = &request.server;
        info!("{} server {}", "Stopping".yellow(), server.name.blue());

        // Remove resources allocated by server from node
        if let Some(node) = server.node.upgrade() {
            node.deallocate(&server.allocation);
        }

        // Send start request to node
        // We do this async because the driver chould be running blocking code like network requests
        let controller = self
            .controller
            .upgrade()
            .expect("The controller is dead while still running code that requires it");
        {
            let server = server.clone();
            controller
                .get_runtime()
                .as_ref()
                .unwrap()
                .spawn_blocking(move || stop_thread(server));
        }

        // Remove server from group and servers list
        if let Some(group) = server.group.upgrade() {
            group.remove_server(server);
        }
        if let Some(controller) = self.controller.upgrade() {
            controller.get_auth().unregister_server(server);
        }
        servers.retain(|handle| !Arc::ptr_eq(handle, server));

        // Remove users connected to the server
        controller.get_users().cleanup_users(server);
        // Remove subscribers from channels
        controller.get_event_bus().cleanup_server(server);

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

    pub fn stop_server_now(&self, server: ServerHandle) {
        self.stop_requests
            .lock()
            .unwrap()
            .push_back(StopRequest { when: None, server });
    }

    pub fn _stop_server(&self, when: Instant, server: ServerHandle) {
        self.stop_requests.lock().unwrap().push_back(StopRequest {
            when: Some(when),
            server,
        });
    }

    pub fn restart_server(&self, server: &ServerHandle) {
        info!("{} server {}", "Restarting".yellow(), server.name.blue());

        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        *server.state.lock().unwrap() = State::Restarting;
        *server.health.lock().unwrap() = Health::new(
            controller.configuration.timings.restart.unwrap(),
            controller.configuration.timings.healthbeat.unwrap(),
        );

        // Send restart request to node
        // We do this async because the driver chould be running blocking code like network requests
        if let Some(controller) = self.controller.upgrade() {
            let server = server.clone();
            let copy = controller.clone();
            controller
                .get_runtime()
                .as_ref()
                .unwrap()
                .spawn_blocking(move || restart_thread(copy, server));
        }

        fn restart_thread(controller: ControllerHandle, server: ServerHandle) {
            if let Some(node) = server.node.upgrade() {
                if let Err(error) = &node.get_inner().restart_server(&server) {
                    error!(
                        "{} to restart server {}: {}",
                        "Failed".red(),
                        server.name.red(),
                        error
                    );
                    controller.get_servers().stop_server_now(server);
                }
            }
        }
    }

    pub fn handle_heart_beat(&self, server: &ServerHandle) {
        debug!("Received heartbeat from server {}", &server.name);

        // Reset health
        server.health.lock().unwrap().reset();

        // Check were the server is in the state machine
        let mut state = server.state.lock().unwrap();
        if *state == State::Starting || *state == State::Restarting {
            *state = State::Preparing;
            info!(
                "The server {} is now {}",
                server.name.blue(),
                "loading".yellow()
            );
        }
    }

    pub fn mark_ready(&self, server: &ServerHandle) {
        if !server.rediness.load(Ordering::Relaxed) {
            debug!("The server {} is {}", server.name.blue(), "ready".green());
            server.rediness.store(true, Ordering::Relaxed);
        }
    }

    pub fn mark_not_ready(&self, server: &ServerHandle) {
        if server.rediness.load(Ordering::Relaxed) {
            debug!(
                "The server {} is {} ready",
                server.name.blue(),
                "no longer".red()
            );
            server.rediness.store(false, Ordering::Relaxed);
        }
    }

    pub fn mark_running(&self, server: &ServerHandle) {
        let mut state = server.state.lock().unwrap();
        if *state == State::Preparing {
            info!(
                "The server {} is now {}",
                server.name.blue(),
                "running".green()
            );
            *state = State::Running;
        }
    }

    pub fn checked_stop_server(&self, server: &ServerHandle) {
        let mut state = server.state.lock().unwrap();
        if *state == State::Running {
            self.mark_not_ready(server);
            *state = State::Stopping;
            self.stop_server_now(server.clone());
        }
    }

    pub fn find_fallback_server(&self, excluded: &ServerHandle) -> Option<ServerHandle> {
        // TODO: Also check if the server have free slots
        self.servers
            .lock()
            .unwrap()
            .iter()
            .filter(|server| {
                !Arc::ptr_eq(server, excluded)
                    && server.allocation.deployment.fallback.enabled
                    && *server.state.lock().unwrap() == State::Running
            })
            .max_by_key(|server| server.allocation.deployment.fallback.priority)
            .cloned()
    }

    fn start_server(
        &self,
        request: &StartRequestHandle,
        allocation: AllocationHandle,
        node: &NodeHandle,
    ) {
        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        info!(
            "{} server {} on node {} listening on port {}",
            "Spinning up".green(),
            request.name.blue(),
            node.name.blue(),
            allocation.primary_address().to_string().blue()
        );
        let server = Arc::new_cyclic(|handle| {
            // Create a token for the server
            let auth = self
                .controller
                .upgrade()
                .expect("WAIT. We are creating a server while the controller is dead?")
                .get_auth()
                .register_server(handle.clone());

            Server {
                name: request.name.clone(),
                uuid: Uuid::new_v4(),
                group: request.group.clone(),
                node: Arc::downgrade(node),
                allocation,
                auth,
                health: Mutex::new(Health::new(
                    controller.configuration.timings.startup.unwrap(),
                    controller.configuration.timings.healthbeat.unwrap(),
                )),
                state: Mutex::new(State::Starting),
                rediness: AtomicBool::new(false),
            }
        });

        if let Some(group) = request.group.upgrade() {
            group.set_active(server.clone(), request);
        }
        self.servers.lock().unwrap().push(server.clone());

        // Send start request to node
        // We do this async because the driver chould be running blocking code like network requests
        if let Some(controller) = self.controller.upgrade() {
            let copy = controller.clone();
            controller
                .get_runtime()
                .as_ref()
                .unwrap()
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
                    controller.get_servers().stop_server_now(server);
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

    /* Auth */
    pub auth: AuthServerHandle,

    /* Health and State of the server */
    pub health: Mutex<Health>,
    pub state: Mutex<State>,
    pub rediness: AtomicBool,
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

pub struct StartRequest {
    pub when: Option<Instant>,
    pub name: String,
    pub group: WeakGroupHandle,
    pub nodes: Vec<WeakNodeHandle>,
    pub resources: Resources,
    pub deployment: Deployment,
    pub priority: i32,
}

pub struct StopRequest {
    pub when: Option<Instant>,
    pub server: ServerHandle,
}

#[derive(PartialEq)]
pub enum State {
    Starting,
    Preparing,
    Restarting,
    Running,
    Stopping,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Resources {
    pub memory: u32,
    pub swap: u32,
    pub cpu: u32,
    pub io: u32,
    pub disk: u32,
    pub addresses: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum Retention {
    #[serde(rename = "temporary")]
    #[default]
    Temporary,
    #[serde(rename = "permanent")]
    Permanent,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FallbackPolicy {
    pub enabled: bool,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Deployment {
    pub settings: Vec<KeyValue>,
    pub environment: Vec<KeyValue>,
    pub disk_retention: Retention,
    pub image: String,

    pub fallback: FallbackPolicy,
}
