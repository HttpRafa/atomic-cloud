use std::time::{Duration, Instant};
use tokio::sync::mpsc::Receiver;
use tokio::time;

use crate::config::Config;
use crate::driver::Drivers;
use crate::group::Groups;
use crate::network::{start_controller_server, NetworkTask};
use crate::node::Nodes;
use crate::server::Servers;

const TICK_RATE: u64 = 1;

pub struct Controller {
    configuration: Config,
    drivers: Drivers,
    nodes: Nodes,
    groups: Groups,
    servers: Servers
}

impl Controller {
    pub async fn new(configuration: Config) -> Self {
        let drivers = Drivers::load_all().await;
        let nodes = Nodes::load_all(&drivers).await;
        let groups = Groups::load_all(&nodes).await;
        Self {
            configuration,
            drivers,
            nodes,
            groups,
            servers: Servers::new()
        }
    }

    pub async fn start(&mut self) {
        let mut network_tasks = start_controller_server(&self.configuration);
        let tick_duration = Duration::from_millis(1000 / TICK_RATE);

        loop {
            let start_time = Instant::now();
            self.tick(&mut network_tasks).await;

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                time::sleep(tick_duration - elapsed_time).await;
            }
        }

        // TODO: Add if the controller can exist by is own: network_handle.abort();
    }

    async fn tick(&mut self, _network_tasks: &mut Receiver<NetworkTask>) {
        // Process network requests
        /*while let Ok(task) = network_tasks.try_recv() {
            match task {
                // Add task handling logic here
                _ => warn!("No task handling implemented"),
            }
        }*/

        // Tick server manager
        self.groups.tick();
    }
}