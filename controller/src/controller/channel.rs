use std::{collections::HashMap, sync::Arc};

use log::error;
use tokio::sync::{mpsc::Sender, Mutex};
use tonic::Status;

use crate::network::server::proto::ChannelMessage;

use super::{
    server::{ServerHandle, WeakServerHandle},
    ControllerHandle,
};

pub type ChannelHandle = Arc<Channel>;

pub struct Channels {
    //controller: WeakControllerHandle,

    /* All active channels */
    channels: Mutex<HashMap<String, ChannelHandle>>,
}

impl Channels {
    pub fn new(/*controller: WeakControllerHandle*/) -> Self {
        Self {
            //controller,
            channels: Mutex::new(HashMap::new()),
        }
    }

    pub fn cleanup_subscribers(controller: &ControllerHandle, server: &ServerHandle) {
        let owned_controller = controller.clone();
        let server = server.clone();
        controller
            .get_runtime()
            .as_ref()
            .unwrap()
            .spawn(async move {
                owned_controller
                    .get_channels()
                    .async_cleanup_subscribers(&server)
                    .await;
            });
    }

    pub async fn async_cleanup_subscribers(&self, server: &ServerHandle) {
        let channels = self.channels.lock().await;
        for channel in channels.values() {
            channel.unsubscribe(server).await;
        }
    }

    pub async fn subscribe_to_channel(&self, name: &str, subscriber: ChannelSubscriber) {
        let mut channels = self.channels.lock().await;
        if let Some(channel) = channels.get(name) {
            channel.subscribe(subscriber).await;
        } else {
            let channel = Channel::new();
            channel.subscribe(subscriber).await;
            channels.insert(name.to_string(), Arc::new(channel));
        }
    }

    pub async fn unsubscribe_from_channel(&self, name: &str, server: &ServerHandle) {
        let channels = self.channels.lock().await;
        if let Some(channel) = channels.get(name) {
            channel.unsubscribe(server).await;
        }
    }

    pub async fn broadcast_to_channel(&self, message: ChannelMessage) -> u32 {
        let channels = self.channels.lock().await;
        if let Some(channel) = channels.get(&message.channel) {
            return channel.send_to_all(message).await;
        }
        0
    }
}

pub struct ChannelSubscriber {
    pub server: WeakServerHandle,
    pub sender: Sender<Result<ChannelMessage, Status>>,
}

impl ChannelSubscriber {
    pub fn new(server: WeakServerHandle, sender: Sender<Result<ChannelMessage, Status>>) -> Self {
        Self { server, sender }
    }
}

pub struct Channel {
    pub subscribers: Mutex<Vec<ChannelSubscriber>>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
        }
    }

    pub async fn subscribe(&self, subscriber: ChannelSubscriber) {
        self.subscribers.lock().await.push(subscriber);
    }

    pub async fn unsubscribe(&self, server: &ServerHandle) {
        self.subscribers.lock().await.retain(|item| {
            if let Some(strong_server) = item.server.upgrade() {
                !Arc::ptr_eq(server, &strong_server)
            } else {
                false
            }
        })
    }

    pub async fn send_to_all(&self, message: ChannelMessage) -> u32 {
        let mut count = 0;
        for subscriber in self.subscribers.lock().await.iter() {
            if let Err(error) = subscriber.sender.send(Ok(message.clone())).await {
                error!("Failed to send message to subscriber: {}", error);
            }
            count += 1;
        }
        count
    }
}
