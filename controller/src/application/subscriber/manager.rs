use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use uuid::Uuid;

use crate::network::client::{ChannelMsg, TransferMsg};

use super::Subscriber;

type SubscriberHolder<A, B> = RwLock<HashMap<A, Vec<Subscriber<B>>>>;

pub struct SubscriberManager {
    transfer: SubscriberHolder<Uuid, TransferMsg>,
    channel: SubscriberHolder<String, ChannelMsg>,
}

impl SubscriberManager {
    pub fn init() -> Self {
        Self {
            transfer: RwLock::new(HashMap::new()),
            channel: RwLock::new(HashMap::new()),
        }
    }

    pub async fn subscribe_transfer(
        &self,
        server: Uuid,
    ) -> ReceiverStream<Result<TransferMsg, Status>> {
        let (subscriber, receiver) = Subscriber::create();
        self.transfer
            .write()
            .await
            .entry(server)
            .or_default()
            .push(subscriber);
        receiver
    }

    pub async fn subscribe_channel(
        &self,
        channel: String,
    ) -> ReceiverStream<Result<ChannelMsg, Status>> {
        let (subscriber, receiver) = Subscriber::create();
        self.channel
            .write()
            .await
            .entry(channel)
            .or_default()
            .push(subscriber);
        receiver
    }

    pub async fn publish_transfer(&self, server: &Uuid, message: TransferMsg) -> u32 {
        let mut count = 0;
        if let Some(subscribers) = self.transfer.read().await.get(server) {
            for subscriber in subscribers {
                subscriber.0.send(Ok(message.clone())).await.unwrap();
                count += 1;
            }
        }
        count
    }

    pub async fn publish_channel(&self, message: ChannelMsg) -> u32 {
        let mut count = 0;
        if let Some(subscribers) = self.channel.read().await.get(&message.channel) {
            for subscriber in subscribers {
                subscriber.0.send(Ok(message.clone())).await.unwrap();
                count += 1;
            }
        }
        count
    }
}

// Ticking
impl SubscriberManager {
    pub async fn tick(&self) -> Result<()> {
        // Cleanup dead subscribers
        Self::cleanup(&self.channel).await;
        Self::cleanup(&self.transfer).await;

        Ok(())
    }

    async fn cleanup<A, B>(holder: &SubscriberHolder<A, B>) {
        holder.write().await.retain(|_, value| {
            value.retain(super::Subscriber::is_alive);
            !value.is_empty()
        });
    }
}
