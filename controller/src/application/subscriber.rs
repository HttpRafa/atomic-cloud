use tokio::sync::mpsc::{channel, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;

pub mod manager;

const SUBSCRIPTION_BUFFER: usize = 64;

pub struct Subscriber<T>(pub Sender<Result<T, Status>>);

impl<T> Subscriber<T> {
    pub fn create() -> (Self, ReceiverStream<Result<T, Status>>) {
        let (subscriber, receiver) = channel(SUBSCRIPTION_BUFFER);
        (Self(subscriber), ReceiverStream::new(receiver))
    }

    pub fn is_alive(&self) -> bool {
        !self.0.is_closed()
    }
}
