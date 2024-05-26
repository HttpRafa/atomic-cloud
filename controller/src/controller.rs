use std::collections::VecDeque;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{error, info, warn};
use tokio::time;
use crate::config::Config;
use crate::driver::Drivers;
use crate::network::start_controller_server;

const TICK_RATE: u64 = 1;

pub struct Controller {
    configuration: Config,
    drivers: Drivers,
    tick_queue: Arc<Mutex<TaskQueue>>
}

impl Controller {
    pub async fn new(configuration: Config) -> Self {
        Controller {
            configuration,
            drivers: Drivers::new(),
            tick_queue: TaskQueue::new_mutex()
        }
    }
    pub async fn start(&mut self) {
        info!("Starting networking stack...");
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
        let mut tasks = self.tick_queue.lock()
            .unwrap_or_else(|error| { error!("Failed to acquire lock of tick queue: {}", error);exit(1); });
        while !tasks.queue.is_empty() {
            let task = &tasks.queue.pop_back().unwrap_or_else(|| { warn!("Failed to get task that should exist"); Task::None });
            match task {
                Task::None => {}
            }
        }
    }
}

pub type AsyncTaskQueue = Arc<Mutex<TaskQueue>>;

pub struct TaskQueue {
    pub(self) queue: VecDeque<Task>
}

impl TaskQueue {
    pub(self) fn new_mutex() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(TaskQueue {
            queue: VecDeque::new()
        }))
    }
}

pub enum Task {
    None
}