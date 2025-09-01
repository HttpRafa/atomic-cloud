use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::Controller,
    network::proto::manage::plugin::{List, Short},
    task::{BoxedAny, GenericTask, network::TonicTask},
};

pub struct GetPluginsTask();

#[async_trait]
impl GenericTask for GetPluginsTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        TonicTask::new_ok(List {
            plugins: controller
                .plugins
                .get_plugins_keys()
                .iter()
                .map(std::convert::Into::into)
                .collect(),
        })
    }
}

impl From<&&String> for Short {
    fn from(plugin: &&String) -> Self {
        Self {
            name: (*plugin).clone(),
        }
    }
}
