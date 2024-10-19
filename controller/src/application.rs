use anyhow::Error;
use auth::Auth;
use colored::Colorize;
use driver::Drivers;
use event::EventBus;
use group::Groups;
use log::info;
use node::Nodes;
use server::Servers;
use user::Users;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Runtime};

use crate::config::Config;
use crate::network::NetworkStack;

pub mod auth;
pub mod driver;
pub mod event;
pub mod group;
pub mod node;
pub mod server;
pub mod user;

static STARTUP_SLEEP: Duration = Duration::from_secs(1);
static SHUTDOWN_WAIT: Duration = Duration::from_secs(10);

const TICK_RATE: u64 = 1;

pub type ControllerHandle = Arc<Controller>;
pub type WeakControllerHandle = Weak<Controller>;

pub struct Controller {
    handle: WeakControllerHandle,

    /* Immutable */
    pub(crate) configuration: Config,
    pub(crate) drivers: Drivers,

    /* Runtime State */
    runtime: Mutex<Option<Runtime>>,
    running: AtomicBool,

    /* Authentication */
    auth: Auth,

    /* Accessed rarely */
    nodes: Mutex<Nodes>,
    groups: Mutex<Groups>,

    /* Accessed frequently */
    servers: Servers,
    users: Users,

    /* Event Bus */
    event_bus: EventBus,
}

impl Controller {
    pub fn new(configuration: Config) -> Arc<Self> {
        Arc::new_cyclic(move |handle| {
            let auth = Auth::load_all();
            let drivers = Drivers::load_all(configuration.identifier.as_ref().unwrap());
            let nodes = Nodes::load_all(&drivers);
            let groups = Groups::load_all(&nodes);
            let servers = Servers::new(handle.clone());
            let users = Users::new(handle.clone());
            let event_bus = EventBus::new(/*handle.clone()*/);
            Self {
                handle: handle.clone(),
                configuration,
                drivers,
                runtime: Mutex::new(Some(
                    Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create Tokio runtime"),
                )),
                running: AtomicBool::new(true),
                auth,
                nodes: Mutex::new(nodes),
                groups: Mutex::new(groups),
                servers,
                users,
                event_bus,
            }
        })
    }

    pub fn start(&self) {
        // Set up signal handlers
        self.setup_interrupts();

        let network_handle = NetworkStack::start(self.handle.upgrade().unwrap());
        let tick_duration = Duration::from_millis(1000 / TICK_RATE);

        // Wait for 1 second before starting the tick loop
        thread::sleep(STARTUP_SLEEP);

        while self.running.load(Ordering::Relaxed) {
            let start_time = Instant::now();
            self.tick();

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                thread::sleep(tick_duration - elapsed_time);
            }
        }

        // Stop all servers
        info!("Stopping all servers...");
        self.servers.stop_all();

        // Stop network stack
        info!("Stopping network stack...");
        network_handle.shutdown();

        // Wait for all tokio task to finish
        info!("Stopping async runtime...");
        (*self.runtime.lock().unwrap())
            .take()
            .unwrap()
            .shutdown_timeout(SHUTDOWN_WAIT);
    }

    pub fn request_stop(&self) {
        info!("Controller stop requested. Stopping...");
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn lock_nodes(&self) -> MutexGuard<Nodes> {
        self.nodes.lock().expect("Failed to get lock to nodes")
    }

    pub fn lock_groups(&self) -> MutexGuard<Groups> {
        self.groups.lock().expect("Failed to get lock to groups")
    }

    pub fn get_auth(&self) -> &Auth {
        &self.auth
    }

    pub fn get_servers(&self) -> &Servers {
        &self.servers
    }

    pub fn get_users(&self) -> &Users {
        &self.users
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn get_runtime(&self) -> MutexGuard<Option<Runtime>> {
        self.runtime.lock().expect("Failed to get lock to runtime")
    }

    fn tick(&self) {
        // Check if all groups have started there servers etc..
        self.lock_groups().tick(&self.servers);

        // Check if all servers have sent their heartbeats and start requested server if we can
        self.servers.tick();

        // Check state of all users
        self.users.tick();
    }

    fn setup_interrupts(&self) {
        // Set up signal handlers
        let controller = self.handle.clone();
        ctrlc::set_handler(move || {
            info!("{} signal received. Stopping...", "Interrupt".red());
            if let Some(controller) = controller.upgrade() {
                controller.request_stop();
            }
        })
        .expect("Failed to set Ctrl+C handler");
    }
}

pub enum CreationResult {
    Created,
    AlreadyExists,
    Denied(Error),
}
