use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{anyhow, Result};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use super::Task;

const TASK_BUFFER: usize = 128;

#[derive(Clone)]
pub struct TaskSender(Arc<AtomicBool>, Sender<Task>);

pub struct TaskManager {
    ready: Arc<AtomicBool>,

    sender: Sender<Task>,
    receiver: Receiver<Task>,
}

impl TaskManager {
    pub fn init() -> Self {
        let (sender, receiver) = channel(TASK_BUFFER);
        Self {
            ready: Arc::new(AtomicBool::new(false)),
            sender,
            receiver,
        }
    }

    pub fn set_ready(&self, ready: bool) {
        self.ready.store(ready, Ordering::Relaxed);
    }

    pub fn get_sender(&self) -> TaskSender {
        TaskSender(self.ready.clone(), self.sender.clone())
    }

    pub async fn recv(&mut self) -> Option<Task> {
        self.receiver.recv().await
    }
}

impl TaskSender {
    pub fn inner(&self) -> Result<&Sender<Task>> {
        if self.0.load(Ordering::Relaxed) {
            Ok(&self.1)
        } else {
            Err(anyhow!("Attempting to use the task system before it is ready or after it has been shut down. Was it used during the initialization or cleanup phase?"))
        }
    }
}
