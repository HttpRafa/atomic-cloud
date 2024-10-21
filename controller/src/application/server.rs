use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, RwLock, RwLockReadGuard, Weak,
    },
    time::{Duration, Instant},
};

use colored::Colorize;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    auth::AuthServerHandle,
    group::WeakGroupHandle,
    node::{AllocationHandle, NodeHandle, WeakNodeHandle},
    ControllerHandle, WeakControllerHandle,
};

pub type ServerHandle = Arc<Server>;
pub type WeakServerHandle = Weak<Server>;
pub type StartRequestHandle = Arc<StartRequest>;

pub struct Servers {
    controller: WeakControllerHandle,

    /* Servers started by this atomic cloud instance */
    servers: RwLock<HashMap<Uuid, ServerHandle>>,

    /* Servers that should be started/stopped next controller tick */
    start_requests: RwLock<VecDeque<StartRequestHandle>>,
    stop_requests: RwLock<VecDeque<StopRequest>>,
}

impl Servers {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            servers: RwLock::new(HashMap::new()),
            start_requests: RwLock::new(VecDeque::new()),
            stop_requests: RwLock::new(VecDeque::new()),
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
            let dead_servers = self.servers.read().unwrap().values().filter(|server| {
                let health = server.health.read().unwrap();
                if health.is_dead() {
                    match *server.state.read().unwrap() {
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
            let mut requests = self.stop_requests.write().unwrap();
            requests.retain(|request| {
                if let Some(when) = request.when {
                    if when > Instant::now() {
                        return true;
                    }
                }

                self.stop_server_nolock(request, &mut self.servers.write().unwrap());
                false
            });
        }

        // Sort requests by priority and process them
        {
            let mut requests = self.start_requests.write().unwrap();
            {
                let contiguous = requests.make_contiguous();
                contiguous.sort_unstable_by_key(|req| req.priority);
                contiguous.reverse();
            }
            requests.retain(|request| {
                if request.canceled.load(Ordering::Relaxed) {
                    debug!(
                        "{} start of server {}",
                        "Canceled".yellow(),
                        request.name.blue()
                    );
                    return false;
                }

                if let Some(when) = request.when {
                    if when > Instant::now() {
                        return true;
                    }
                }

                if request.nodes.is_empty() {
                    warn!(
                        "{} to allocate resources for server {} because no nodes were specified",
                        "Failed".red(),
                        request.name.red()
                    );
                    return true;
                }

                // Collect and sort nodes by the number of allocations
                for node in &request.nodes {
                    let node = node.upgrade().unwrap();
                    // Try to allocate resources on nodes
                    if let Ok(allocation) = node.allocate(request) {
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
        self.start_requests.write().unwrap().push_back(arc.clone());
        arc
    }

    pub fn stop_all_instant(&self) {
        self.servers
            .write()
            .unwrap()
            .drain()
            .for_each(|(_, server)| {
                self.stop_server_internal(&StopRequest { when: None, server });
            });
    }

    pub fn stop_all_on_node(&self, node: &NodeHandle) {
        self.servers
            .read()
            .unwrap()
            .values()
            .filter(|server| Arc::ptr_eq(&server.node.upgrade().unwrap(), node))
            .for_each(|server| {
                self.stop_server_now(server.clone());
            });
    }

    fn stop_server_nolock(&self, request: &StopRequest, servers: &mut HashMap<Uuid, ServerHandle>) {
        self.stop_server_internal(request);
        servers.remove(&request.server.uuid);
    }

    fn stop_server_internal(&self, request: &StopRequest) {
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
        if let Some(group) = &server.group {
            group.remove_server(server);
        }
        if let Some(controller) = self.controller.upgrade() {
            controller.get_auth().unregister_server(server);
        }

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
            .write()
            .unwrap()
            .push_back(StopRequest { when: None, server });
    }

    pub fn _stop_server(&self, when: Instant, server: ServerHandle) {
        self.stop_requests.write().unwrap().push_back(StopRequest {
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

        *server.state.write().unwrap() = State::Restarting;
        *server.health.write().unwrap() = Health::new(
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
        server.health.write().unwrap().reset();

        // Check were the server is in the state machine
        let mut state = server.state.write().unwrap();
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
        let mut state = server.state.write().unwrap();
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
        let mut state = server.state.write().unwrap();
        if *state != State::Stopping {
            self.mark_not_ready(server);
            *state = State::Stopping;
            self.stop_server_now(server.clone());
        }
    }

    pub fn find_fallback_server(&self, excluded: &ServerHandle) -> Option<ServerHandle> {
        // TODO: Also check if the server have free slots
        self.servers
            .read()
            .unwrap()
            .values()
            .filter(|server| {
                !Arc::ptr_eq(server, excluded)
                    && server.allocation.deployment.fallback.enabled
                    && *server.state.read().unwrap() == State::Running
            })
            .max_by_key(|server| server.allocation.deployment.fallback.priority)
            .cloned()
    }

    pub fn get_server(&self, uuid: Uuid) -> Option<ServerHandle> {
        self.servers.read().unwrap().get(&uuid).cloned()
    }

    pub fn get_servers(&self) -> RwLockReadGuard<HashMap<Uuid, ServerHandle>> {
        self.servers.read().expect("Failed to lock servers")
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
                connected_users: AtomicU32::new(0),
                auth,
                health: RwLock::new(Health::new(
                    controller.configuration.timings.startup.unwrap(),
                    controller.configuration.timings.healthbeat.unwrap(),
                )),
                state: RwLock::new(State::Starting),
                rediness: AtomicBool::new(false),
                flags: Flags {
                    stop: RwLock::new(None),
                },
            }
        });

        if let Some(group) = &request.group {
            group.set_active(server.clone(), request);
        }
        self.servers
            .write()
            .unwrap()
            .insert(server.uuid, server.clone());

        // Print server information to the console for debugging
        debug!("{}", "-----------------------------------".red());
        debug!("{}", "New server added to controller".red());
        debug!("{}{}", "Name: ".red(), server.name.red());
        debug!("{}{}", "UUID: ".red(), server.uuid.to_string().red());
        debug!("{}{}", "Token: ".red(), server.auth.token.red());
        debug!("{}", "-----------------------------------".red());

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
    pub group: Option<GroupInfo>,
    pub node: WeakNodeHandle,
    pub allocation: AllocationHandle,

    /* Users */
    pub connected_users: AtomicU32,

    /* Auth */
    pub auth: AuthServerHandle,

    /* Health and State of the server */
    pub health: RwLock<Health>,
    pub state: RwLock<State>,
    pub flags: Flags,
    pub rediness: AtomicBool,
}

impl Server {
    pub fn get_player_count(&self) -> u32 {
        self.connected_users.load(Ordering::Relaxed)
    }
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
    pub canceled: AtomicBool,
    pub when: Option<Instant>,
    pub name: String,
    pub group: Option<GroupInfo>,
    pub nodes: Vec<WeakNodeHandle>,
    pub resources: Resources,
    pub deployment: Deployment,
    pub priority: i32,
}

pub struct StopRequest {
    pub when: Option<Instant>,
    pub server: ServerHandle,
}

#[derive(PartialEq, Clone)]
pub enum State {
    Starting,
    Preparing,
    Restarting,
    Running,
    Stopping,
}

pub struct Flags {
    /* Required for the group system */
    pub stop: RwLock<Option<Instant>>,
}

#[derive(Clone)]
pub struct GroupInfo {
    pub server_id: usize,
    pub group: WeakGroupHandle,
}

impl GroupInfo {
    pub fn remove_server(&self, server: &ServerHandle) {
        if let Some(group) = self.group.upgrade() {
            group.remove_server(server);
        }
    }

    pub fn set_active(&self, server: ServerHandle, request: &StartRequestHandle) {
        if let Some(group) = self.group.upgrade() {
            group.set_server_active(server, request);
        }
    }
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
