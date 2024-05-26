use std::collections::VecDeque;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{error, warn};
use tokio::time;
use crate::config::Config;
use crate::driver::Drivers;
use crate::network::start_controller_server;
use crate::node::Nodes;

const TICK_RATE: u64 = 1;

pub struct Controller {
    configuration: Config,
    drivers: Drivers,
    nodes: Nodes,
    tick_queue: Arc<Mutex<TaskQueue>>,
}

impl Controller {
    pub async fn new(configuration: Config) -> Self {
        let drivers = Drivers::load_all();
        let nodes = Nodes::load_all(&drivers);
        Controller {
            configuration,
            drivers,
            nodes,
            tick_queue: TaskQueue::new_mutex(),
        }
    }

    pub async fn start(&mut self) {
        start_controller_server(&self.configuration);

        let tick_duration = Duration::from_millis(1000 / TICK_RATE);
        loop {
            let start_time = Instant::now();

            self.tick();

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                time::sleep(tick_duration - elapsed_time).await;
            }
        }
    }

    fn tick(&mut self) {
        let mut tasks = self.tick_queue.lock().unwrap_or_else(|error| {
            error!("Failed to acquire lock of tick queue: {}", error);
            exit(1);
        });

        while let Some(task) = tasks.queue.pop_back() {
            match task {
                // Add task handling logic here
                _ => warn!("No task handling implemented"),
            }
        }
    }
}

pub type AsyncTaskQueue = Arc<Mutex<TaskQueue>>;

pub struct TaskQueue {
    pub(self) queue: VecDeque<Task>,
}

impl TaskQueue {
    pub(self) fn new_mutex() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(TaskQueue {
            queue: VecDeque::new(),
        }))
    }
}

pub enum Task {
}