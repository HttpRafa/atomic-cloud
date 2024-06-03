use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use log::warn;
use tokio::sync::{Mutex, MutexGuard};
use tokio::time;

use crate::config::Config;
use crate::network::start_controller_server;
use crate::controller::driver::Drivers;
use crate::controller::group::Groups;
use crate::controller::node::Nodes;
use crate::controller::server::Servers;

pub mod node;
pub mod group;
pub mod server;
pub mod driver;

const TICK_RATE: u64 = 1;

type ControllerHandle = Weak<Controller>;

pub struct Controller {
    handle: ControllerHandle,

    /* Immutable */
    pub(crate) configuration: Config,
    pub(crate) drivers: Drivers,

    /* Runtime State */
    running: AtomicBool,

    /* Mutable | This can be changed by the user at runtime */
    nodes: Mutex<Nodes>,
    groups: Mutex<Groups>,
    servers: Mutex<Servers>,
}

impl Controller {
    pub async fn new(configuration: Config) -> Arc<Self> {
        let drivers = Drivers::load_all().await;
        let nodes = Nodes::load_all(&drivers).await;
        let groups = Groups::load_all(&nodes).await;
        let servers = Servers::new();
        Arc::new_cyclic(move |handle| Self {
            handle: handle.clone(),
            configuration,
            drivers,
            running: AtomicBool::new(true),
            nodes: Mutex::new(nodes),
            groups: Mutex::new(groups),
            servers: Mutex::new(servers),
        })
    }

    pub async fn start(&self) {
        let network_handle = start_controller_server(self.handle.upgrade().unwrap());
        let tick_duration = Duration::from_millis(1000 / TICK_RATE);

        while self.running.load(Ordering::Relaxed) {
            let start_time = Instant::now();
            self.tick().await;

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                time::sleep(tick_duration - elapsed_time).await;
            }
        }

        // Stop network stack
        network_handle.abort();
    }

    pub fn request_stop(&self) {
        warn!("Controller stop requested. Stopping...");
        self.running.store(false, Ordering::Relaxed);
    }

    pub async fn request_nodes(&self) -> MutexGuard<Nodes> {
        self.nodes.lock().await
    }

    pub async fn request_groups(&self) -> MutexGuard<Groups> {
        self.groups.lock().await
    }

    async fn tick(&self) {
        // Tick server manager
        self.request_groups().await.tick();
    }
}

pub enum CreationResult {
    Created,
    AlreadyExists,
    Denied(String),
}