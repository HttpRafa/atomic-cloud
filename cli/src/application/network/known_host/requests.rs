use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    vec,
};

use tokio::sync::oneshot::{channel, Sender};

use super::KnownHost;

#[derive(Debug)]
pub struct RequestTracker {
    queue: RwLock<VecDeque<TrustRequest>>,
    open: RwLock<Vec<TrustRequest>>,
    watcher: RwLock<Vec<Sender<()>>>,
}

impl Default for RequestTracker {
    fn default() -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            open: RwLock::new(vec![]),
            watcher: RwLock::new(vec![]),
        }
    }
}

impl RequestTracker {
    pub fn enqueue(&self, request: TrustRequest) {
        self.queue
            .write()
            .expect("Failed to lock queue")
            .push_back(request);
    }

    pub fn dequeue(&self) -> Option<TrustRequest> {
        self.queue
            .write()
            .expect("Failed to lock queue")
            .pop_front()
            .inspect(|request| {
                self.open
                    .write()
                    .expect("Failed to lock open cache")
                    .push(request.clone());
            })
    }

    pub async fn wait_for_empty(&self) {
        let (sender, receiver) = channel();
        self.watcher
            .write()
            .expect("Failed to lock watchers")
            .push(sender);
        let _ = receiver.await;
    }

    pub fn cleanup(&self) {
        let mut cache = self.open.write().expect("Failed to lock open cache");
        cache.retain(|request| !request.0 .0.load(Ordering::Relaxed));
        if cache.is_empty() {
            for sender in self
                .watcher
                .write()
                .expect("Failed to lock watchers")
                .drain(..)
            {
                let _ = sender.send(());
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TrustRequest(Arc<(AtomicBool, KnownHost)>);

impl TrustRequest {
    pub fn new(host: KnownHost) -> Self {
        Self(Arc::new((AtomicBool::new(false), host)))
    }

    pub fn complete(&self) {
        self.0 .0.store(true, Ordering::Relaxed);
    }

    pub fn get_host(&self) -> &KnownHost {
        &self.0 .1
    }
}

#[derive(Debug)]
pub enum TrustResult {
    Trusted,
    Declined,
}
