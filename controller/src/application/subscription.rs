use tokio::sync::mpsc::Sender;

pub mod manager;

pub type Subscription<T> = Sender<T>;