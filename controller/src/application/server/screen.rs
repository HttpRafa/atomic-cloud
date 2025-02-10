use anyhow::Result;
use tokio::task::JoinHandle;

pub mod manager;

pub trait GenericScreen {
    fn is_supported(&self) -> bool;
    fn pull(&self) -> JoinHandle<Result<Vec<String>, PullError>>;
}

pub enum PullError {
    Unsupported,
    Error(anyhow::Error),
}