use anyhow::Result;
use common::network::HostAndPort;
use tokio::task::JoinHandle;
use tonic::async_trait;
use url::Url;

use super::{
    node::Capabilities,
    server::{manager::StartRequest, screen::GenericScreen, Server},
};

pub mod manager;
mod runtime;

pub type BoxedPlugin = Box<dyn GenericPlugin + Send + Sync>;
pub type BoxedNode = Box<dyn GenericNode + Send + Sync>;
pub type BoxedScreen = Box<dyn GenericScreen + Send + Sync>;

#[async_trait]
pub trait GenericPlugin {
    async fn init(&self) -> Result<Information>;
    async fn init_node(
        &self,
        name: &str,
        capabilities: &Capabilities,
        controller: &Url,
    ) -> Result<BoxedNode>;

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
    fn start(&self, server: &Server) -> JoinHandle<Result<BoxedScreen>>;
    fn restart(&self, server: &Server) -> JoinHandle<Result<()>>;
    fn stop(&self, server: &Server) -> JoinHandle<Result<()>>;
}

pub struct Information {
    authors: Vec<String>,
    version: String,
    ready: bool,
}
