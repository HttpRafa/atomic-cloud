use std::time::{Duration, Instant};
use log::warn;
use tokio::sync::mpsc::Receiver;
use tokio::time;

use crate::config::Config;
use crate::driver::Drivers;
use crate::network::{NetworkTask, start_controller_server};
use crate::node::Nodes;

const TICK_RATE: u64 = 5;

pub struct Controller {
    configuration: Config,
    drivers: Drivers,
    nodes: Nodes,
}

impl Controller {
    pub async fn new(configuration: Config) -> Self {
        let drivers = Drivers::load_all().await;
        let nodes = Nodes::load_all(&drivers).await;
        Self {
            configuration,
            drivers,
            nodes,
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
    }

    async fn tick(&mut self, network_tasks: &mut Receiver<NetworkTask>) {
        // Process network requests
        while let Ok(task) = network_tasks.try_recv() {
            match task {
                // Add task handling logic here
                //_ => warn!("No task handling implemented"),
            }
        }
    }
}