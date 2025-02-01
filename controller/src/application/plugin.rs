use anyhow::Result;
use tokio::task::JoinHandle;
use tonic::async_trait;

use super::node::{Capabilities, RemoteController};

pub mod manager;
mod runtime;

pub type WrappedPlugin = Box<dyn GenericPlugin>;
pub type WrappedNode = Box<dyn GenericNode>;

#[async_trait]
pub trait GenericPlugin {
    async fn init(&self) -> Result<Information>;
    async fn init_node(
        &self,
        name: &str,
        capabilities: &Capabilities,
        remote: &RemoteController,
    ) -> Result<WrappedNode>;

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
