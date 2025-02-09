use anyhow::Result;
use tonic::async_trait;

use crate::{
    application::{user::User, Controller},
    network::proto::manage::user::{Item, List},
    task::{BoxedAny, GenericTask, Task},
};

pub struct GetUsersTask();

#[async_trait]
impl GenericTask for GetUsersTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            users: controller
                .users
                .get_users()
                .iter()
                .map(std::convert::Into::into)
                .collect(),
        })
    }
}

impl From<&&User> for Item {
    fn from(user: &&User) -> Self {
        Self {
            id: user.id().uuid().to_string(),
            name: user.id().name().clone(),
        }
    }
}
