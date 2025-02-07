use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::{auth::Authorization, Controller},
    task::{BoxedAny, GenericTask, Task},
};

pub struct BeatTask {
    pub auth: Authorization,
}

#[async_trait]
impl GenericTask for BeatTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let server = match self
            .auth
            .get_server()
            .and_then(|server| controller.servers.get_server_mut(server.uuid()))
        {
            Some(server) => server,
            None => return Task::new_link_error(),
        };
        server.heart_mut().beat();
        Task::new_empty()
    }
}
