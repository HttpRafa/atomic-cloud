use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    network::proto::manage::plugin::List,
    task::{BoxedAny, GenericTask, Task},
};

pub struct GetPluginsTask();

#[async_trait]
impl GenericTask for GetPluginsTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            plugins: controller
                .plugins
                .get_plugins_keys()
                .iter()
                .map(|plugin| (*plugin).clone())
                .collect(),
        })
    }
}
