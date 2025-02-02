use anyhow::Result;
use common::network::HostAndPort;
use tokio::task::JoinHandle;
use tonic::async_trait;

use super::{
    node::{Capabilities, RemoteController},
    server::{manager::StartRequest, Server},
};

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

    /* Shutdown */
    fn shutdown(&self) -> JoinHandle<Result<()>>;

    /* Ticking */
    fn tick(&self) -> JoinHandle<Result<()>>;
}

pub trait GenericNode {
    /* Ticking */
    fn tick(&self) -> JoinHandle<Result<()>>;

    /* Prepare */
    fn allocate(&self, request: &StartRequest) -> JoinHandle<Result<Vec<HostAndPort>>>;
    fn free(&self, ports: &[HostAndPort]) -> JoinHandle<Result<()>>;

    /* Servers */
    fn start(&self, token: &str, server: &Server) -> JoinHandle<Result<()>>;
    fn restart(&self, server: &Server) -> JoinHandle<Result<()>>;
    fn stop(&self, server: &Server) -> JoinHandle<Result<()>>;
}

pub struct Information {
    authors: Vec<String>,
    version: String,
    ready: bool,
}
