use anyhow::Result;
use tokio::task::JoinHandle;
use tonic::async_trait;
use url::Url;

use crate::application::plugin::{BoxedNode, GenericPlugin, Information, Capabilities, runtime::wasmite::Plugin};

#[async_trait]
impl GenericPlugin for Plugin {
    async fn init(&self) -> Result<Information> {
        unimplemented!()
    }

    async fn init_node(
        &self,
        name: &str,
        capabilities: &Capabilities,
        controller: &Url,
    ) -> Result<BoxedNode> {
        unimplemented!()
    }

    fn shutdown(&self) -> JoinHandle<Result<()>> {
        unimplemented!()
    }

    fn tick(&self) -> JoinHandle<Result<()>> {
        unimplemented!()
    }

    async fn cleanup(&mut self) -> Result<()> {
        unimplemented!()
    }
}