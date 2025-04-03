use std::{collections::HashMap, hash::Hash};

use tokio::sync::RwLock;

use super::Subscriber;

pub struct Watcher<A: Eq + Hash, B>(
    RwLock<Vec<Subscriber<B>>>,
    RwLock<HashMap<A, Vec<Subscriber<B>>>>,
);

impl<A: Eq + Hash, B: Clone> Watcher<A, B> {
    pub fn new() -> Self {
        Self(RwLock::new(Vec::new()), RwLock::new(HashMap::new()))
    }

    pub async fn publish(&self, message: B) -> u32 {
        let mut count = 0;
        for subscriber in self.0.read().await.iter() {
            if subscriber.send_message(message.clone()).await {
                count += 1;
            }
        }
        count
    }

    pub async fn publish_to_scope(&self, scope: &A, message: B) -> u32 {
        let mut count = 0;
        if let Some(subscribers) = self.1.read().await.get(scope) {
            for subscriber in subscribers {
                if subscriber.send_message(message.clone()).await {
                    count += 1;
                }
            }
        }
        count += self.publish(message).await;
        count
    }

    pub async fn subscribe_to_scope(&self, scope: A, subscriber: Subscriber<B>) {
        self.1
            .write()
            .await
            .entry(scope)
            .or_insert_with(Vec::new)
            .push(subscriber);
    }

    pub async fn subscribe(&self, subscriber: Subscriber<B>) {
        self.0.write().await.push(subscriber);
    }

    pub async fn cleanup(&self) {
        self.0.write().await.retain(Subscriber::is_alive);
        self.1.write().await.retain(|_, subscribers| {
            subscribers.retain(Subscriber::is_alive);
            !subscribers.is_empty()
        });
    }
}
