use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::Error;
use log::warn;
use server::Servers;
use tokio::runtime::{Builder, Runtime};

use crate::config::Config;
use crate::network::NetworkStack;
use crate::controller::driver::Drivers;
use crate::controller::group::Groups;
use crate::controller::node::Nodes;

pub mod node;
pub mod group;
pub mod server;
pub mod driver;

const TICK_RATE: u64 = 1;

pub type ControllerHandle = Arc<Controller>;
pub type WeakControllerHandle = Weak<Controller>;

pub struct Controller {
    handle: WeakControllerHandle,

    /* Immutable */
    pub(crate) configuration: Config,
    pub(crate) drivers: Drivers,

    /* Runtime State */
    runtime: Runtime,
    running: AtomicBool,

    /* Accessed rarely */
    nodes: Mutex<Nodes>,
    groups: Mutex<Groups>,

    /* Accessed frequently */
    servers: Servers,
}

impl Controller {
    pub fn new(configuration: Config) -> Arc<Self> {
        let drivers = Drivers::load_all();
        let nodes = Nodes::load_all(&drivers);
        let groups = Groups::load_all(&nodes);
        let servers = Servers::new();
        Arc::new_cyclic(move |handle| {
            Self {
                handle: handle.clone(),
                configuration,
                drivers,
                runtime: Builder::new_multi_thread().enable_all().build().expect("Failed to create Tokio runtime"),
                running: AtomicBool::new(true),
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
        thread::sleep(Duration::from_secs(1));

        while self.running.load(Ordering::Relaxed) {
            let start_time = Instant::now();
            self.tick();

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                thread::sleep(tick_duration - elapsed_time);
            }
        }

        // Stop network stack
        network_handle.shutdown();
    }

    pub fn request_stop(&self) {
        warn!("Controller stop requested. Stopping...");
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn lock_nodes(&self) -> MutexGuard<Nodes> {
        self.nodes.lock().expect("Failed to get lock to nodes")
    }

    pub fn lock_groups(&self) -> MutexGuard<Groups> {
        self.groups.lock().expect("Failed to get lock to groups")
    }

    pub fn get_servers(&self) -> &Servers {
        &self.servers
    }

    pub fn get_runtime(&self) -> &Runtime {
        &self.runtime
    }

    fn tick(&self) {
        let servers = self.get_servers();
        // Check if all groups have started there servers etc..
        self.lock_groups().tick(servers);

        // Check if all servers have sent their heartbeats and start requested server if we can
        servers.tick(&self.handle);
    }
}

pub enum CreationResult {
    Created,
    AlreadyExists,
    Denied(Error),
}