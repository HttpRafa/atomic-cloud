use anyhow::Error;
use auth::Auth;
use colored::Colorize;
use log::info;
use server::Servers;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Runtime};

use crate::config::Config;
use crate::controller::driver::Drivers;
use crate::controller::group::Groups;
use crate::controller::node::Nodes;
use crate::network::NetworkStack;

pub mod driver;
pub mod group;
pub mod node;
pub mod server;
mod auth;

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
}

impl Controller {
    pub fn new(configuration: Config) -> Arc<Self> {
        Arc::new_cyclic(move |handle| {
            let auth = Auth::load_all();
            let drivers = Drivers::load_all(configuration.identifier.as_ref().unwrap());
            let nodes = Nodes::load_all(&drivers);
            let groups = Groups::load_all(&nodes);
            let servers = Servers::new(handle.clone());
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
            }
        })
    }

    pub fn start(&self) {
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
        info!("{} all servers...", "Stopping".yellow());
        self.servers.stop_all();

        // Stop network stack
        info!("{} network stack...", "Stopping".yellow());
        network_handle.shutdown();

        // Wait for all tokio task to finish
        info!("{} async runtime...", "Stopping".yellow());
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

    pub fn get_runtime(&self) -> MutexGuard<Option<Runtime>> {
        self.runtime.lock().expect("Failed to get lock to runtime")
    }

    fn tick(&self) {
        let servers = self.get_servers();
        // Check if all groups have started there servers etc..
        self.lock_groups().tick(servers);

        // Check if all servers have sent their heartbeats and start requested server if we can
        servers.tick();
    }
}

pub enum CreationResult {
    Created,
    AlreadyExists,
    Denied(Error),
}
