use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};
use tokio::time;

use crate::config::Config;
use crate::driver::Drivers;
use crate::group::Groups;
use crate::network::start_controller_server;
use crate::node::Nodes;
use crate::server::Servers;

const TICK_RATE: u64 = 1;

pub type WeakController = Weak<Controller>;

pub struct Controller {
    handle: WeakController,

    pub configuration: Config,
    pub drivers: Drivers,
    pub nodes: Nodes,
    pub groups: Groups,
    pub servers: Servers
}

impl Controller {
    pub async fn new(configuration: Config) -> Arc<Self> {
        let drivers = Drivers::load_all().await;
        let nodes = Nodes::load_all(&drivers).await;
        let groups = Groups::load_all(&nodes).await;
        Arc::new_cyclic(move |handle| Self {
            handle: handle.clone(),
            configuration,
            drivers,
            nodes,
            groups,
            servers: Servers::new()
        })
    }

    pub async fn start(&self) {
        let _network_handle = start_controller_server(self.handle.upgrade().unwrap());
        let tick_duration = Duration::from_millis(1000 / TICK_RATE);

        loop {
            let start_time = Instant::now();
            self.tick().await;

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                time::sleep(tick_duration - elapsed_time).await;
            }
        }

        // TODO: Add if the controller can exist by is own: network_handle.abort();
    }

    async fn tick(&self) {
        // Tick server manager
        self.servers.tick();
    }
}