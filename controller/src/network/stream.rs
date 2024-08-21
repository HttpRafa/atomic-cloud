use std::{
    pin::Pin,
    sync::mpsc::{Receiver, TryRecvError},
    task::{Context, Poll},
};

use tokio_stream::Stream;

pub struct StdReceiverStream<T> {
    receiver: Receiver<T>,
}

impl<T> Stream for StdReceiverStream<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.try_recv() {
            Ok(item) => Poll::Ready(Some(item)),
            Err(TryRecvError::Empty) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }
}

impl<T> StdReceiverStream<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        Self { receiver }
    }
}
