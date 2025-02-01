use anyhow::Result;
use tonic::async_trait;

use crate::application::Controller;

pub type WrappedTask = Box<dyn Task>;

#[async_trait]
pub trait Task {
    async fn run(&mut self, controller: &mut Controller) -> Result<()>;
}
