use anyhow::Result;
use tonic::{Status, async_trait};
use uuid::Uuid;

use crate::{
    application::{
        Controller,
        auth::{ActionResult, Authorization},
        server::NameAndUuid,
        user::CurrentServer,
    },
    network::proto::common::common_user::{Item, List},
    task::{BoxedAny, GenericTask, network::TonicTask},
};

pub struct UserConnectedTask(pub Authorization, pub NameAndUuid);
pub struct UserDisconnectedTask(pub Authorization, pub Uuid);
pub struct GetUserTask(pub Uuid);
pub struct GetUserFromNameTask(pub String);
pub struct GetUsersTask;
pub struct UserCountTask;

#[async_trait]
impl GenericTask for UserConnectedTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = self
            .0
            .get_server()
            .and_then(|server| controller.servers.get_server_mut(server.uuid()))
        else {
            return TonicTask::new_link_error();
        };
        controller.users.user_connected(server, self.1.clone());
        TonicTask::new_empty()
    }
}

#[async_trait]
impl GenericTask for UserDisconnectedTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(server) = self
            .0
            .get_server()
            .and_then(|server| controller.servers.get_server_mut(server.uuid()))
        else {
            return TonicTask::new_link_error();
        };
        if controller.users.user_disconnected(server, &self.1) == ActionResult::Denied {
            return TonicTask::new_permission_error("You are not allowed to disconnect this user");
        }
        TonicTask::new_empty()
    }
}

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
