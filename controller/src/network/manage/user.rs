use anyhow::Result;
use tonic::{async_trait, Status};
use uuid::Uuid;

use crate::{
    application::{user::CurrentServer, Controller},
    network::proto::common::common_user::{Item, List},
    task::{network::TonicTask, BoxedAny, GenericTask},
};

pub struct GetUserTask(pub Uuid);
pub struct GetUserFromNameTask(pub String);
pub struct GetUsersTask;
pub struct UserCountTask;

#[async_trait]
impl GenericTask for GetUserTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(user) = controller.users.get_user(&self.0) else {
            return TonicTask::new_err(Status::not_found("User not found"));
        };

        TonicTask::new_ok(Item {
            name: user.id().name().clone(),
            id: user.id().uuid().to_string(),
            server: if let CurrentServer::Connected(server) = user.server() {
                Some(server.uuid().to_string())
            } else {
                None
            },
        })
    }
}

#[async_trait]
impl GenericTask for GetUserFromNameTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(user) = controller.users.get_user_from_name(&self.0) else {
            return TonicTask::new_err(Status::not_found("User not found"));
        };

        TonicTask::new_ok(Item {
            name: user.id().name().clone(),
            id: user.id().uuid().to_string(),
            server: if let CurrentServer::Connected(server) = user.server() {
                Some(server.uuid().to_string())
            } else {
                None
            },
        })
    }
}

#[async_trait]
impl GenericTask for GetUsersTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        TonicTask::new_ok(List {
            users: controller
                .users
                .get_users()
                .iter()
                .map(|user| Item {
                    name: user.id().name().clone(),
                    id: user.id().uuid().to_string(),
                    server: if let CurrentServer::Connected(server) = user.server() {
                        Some(server.uuid().to_string())
                    } else {
                        None
                    },
                })
                .collect(),
        })
    }
}

#[async_trait]
impl GenericTask for UserCountTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        TonicTask::new_ok(controller.users.get_user_count())
    }
}
