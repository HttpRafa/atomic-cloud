use anyhow::Result;
use tokio::task::JoinHandle;

pub mod manager;
mod runtime;

pub type WrappedPlugin = Box<dyn GenericPlugin>;
pub type WrappedNode = Box<dyn GenericNode>;

pub trait GenericPlugin {
    fn init(&self) -> JoinHandle<Result<Information>>;

    /* Ticking */
    fn tick(&self) -> JoinHandle<Result<()>>;
}

pub trait GenericNode {
    /* Ticking */
    fn tick(&self) -> JoinHandle<Result<()>>;
}

pub struct Information {
    authors: Vec<String>,
    version: String,
    ready: bool,
}
