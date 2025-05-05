use anyhow::Result;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

pub mod manager;
pub mod watcher;

const SUBSCRIPTION_BUFFER: usize = 128;

enum Dispatch<T> {
    Network(Sender<Result<T, Status>>),
    Plugin(Sender<Result<T>>),
}

pub struct Subscriber<T>(Dispatch<T>);

impl<T> Subscriber<T> {
    pub fn create_network() -> (Self, ReceiverStream<Result<T, Status>>) {
        let (sender, receiver) = channel(SUBSCRIPTION_BUFFER);
        (
            Self(Dispatch::Network(sender)),
            ReceiverStream::new(receiver),
        )
    }

    pub fn create_plugin() -> (Self, Receiver<Result<T>>) {
        let (sender, receiver) = channel(SUBSCRIPTION_BUFFER);
        (Self(Dispatch::Plugin(sender)), receiver)
    }

    pub async fn send_network(&self, data: Result<T, Status>) -> bool {
        match &self.0 {
            Dispatch::Network(sender) => {
                if let Err(_) = sender.send(data).await {
                    // Channel closed
                    false
                } else {
                    true
                }
            }
            Dispatch::Plugin(_) => false,
        }
    }

    pub async fn send_message(&self, message: T) -> bool {
        match &self.0 {
            Dispatch::Network(sender) => {
                if let Err(_) = sender.send(Ok(message)).await {
                    // Channel closed
                    false
                } else {
                    true
                }
            }
            Dispatch::Plugin(sender) => {
                if let Err(_) = sender.send(Ok(message)).await {
                    // Channel closed
                    false
                } else {
                    true
                }
            }
        }
    }

    pub fn is_alive(&self) -> bool {
        match &self.0 {
            Dispatch::Network(sender) => !sender.is_closed(),
            Dispatch::Plugin(sender) => !sender.is_closed(),
        }
    }
}
