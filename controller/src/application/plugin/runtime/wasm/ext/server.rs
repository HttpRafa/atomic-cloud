use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::{
        Controller,
        plugin::runtime::wasm::{
            PluginState,
            generated::{
                exports::plugin::system::bridge,
                plugin::system::{self, types::ErrorMessage},
            },
        },
    },
    task::{BoxedAny, GenericTask, plugin::PluginTask},
};

impl system::server::Host for PluginState {
    async fn get_server(
        &mut self,
        uuid: String,
    ) -> Result<Result<Option<bridge::Server>, ErrorMessage>> {
        let Ok(uuid) = Uuid::parse_str(&uuid) else {
            return Ok(Err("Failed to parse provided uuid".to_string()));
        };

        Ok(Ok(PluginTask::execute::<Option<bridge::Server>, _>(
            &self.tasks,
            GetServerTask(uuid),
        )
        .await?))
    }
}

pub struct GetServerTask(pub Uuid);

#[async_trait]
impl GenericTask for GetServerTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = controller.servers.get_server(&self.0) else {
            return PluginTask::new_ok(None::<bridge::Server>);
        };

        PluginTask::new_ok(Some::<bridge::Server>(server.into()))
    }
}
