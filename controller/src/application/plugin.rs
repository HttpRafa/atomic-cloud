use anyhow::Result;
use bitflags::bitflags;
use common::network::HostAndPort;
use tokio::task::JoinHandle;
use tonic::async_trait;
use url::Url;

use super::{
    node::Capabilities,
    server::{guard::Guard, manager::StartRequest, screen::BoxedScreen, Server},
};

pub mod manager;
mod runtime;

pub type BoxedPlugin = Box<dyn GenericPlugin + Send + Sync>;
pub type BoxedNode = Box<dyn GenericNode + Send + Sync>;

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

    /* Management */
    async fn cleanup(&mut self) -> Result<()>;
}

#[async_trait]
pub trait GenericNode {
    /* Ticking */
    fn tick(&self) -> JoinHandle<Result<()>>;

    /* Prepare */
    fn allocate(&self, request: &StartRequest) -> JoinHandle<Result<Vec<HostAndPort>>>;
    fn free(&self, ports: &[HostAndPort]) -> JoinHandle<Result<()>>;

    /* Servers */
    fn start(&self, server: &Server) -> JoinHandle<Result<BoxedScreen>>;
    fn restart(&self, server: &Server) -> JoinHandle<Result<()>>;
    fn stop(&self, server: &Server, guard: Guard) -> JoinHandle<Result<()>>;

    /* Memory */
    async fn cleanup(&mut self) -> Result<()>;
}

pub struct Information {
    authors: Vec<String>,
    version: String,
    #[allow(unused)]
    features: Features,
    ready: bool,
}

bitflags! {
    pub struct Features: u32 {
        const NODE = 1;
        const ALL = Self::NODE.bits();
    }
}
