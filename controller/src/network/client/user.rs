use anyhow::Result;
use tonic::{async_trait, Status};
use uuid::Uuid;

use crate::{
    application::{
        auth::{ActionResult, Authorization},
        server::NameAndUuid,
        user::CurrentServer,
        Controller,
    },
    network::proto::common::common_user::{Item, List},
    task::{BoxedAny, GenericTask, Task},
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
            return Task::new_link_error();
        };
        controller.users.user_connected(server, self.1.clone());
        Task::new_empty()
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
            return Task::new_link_error();
        };
        if controller.users.user_disconnected(server, &self.1) == ActionResult::Denied {
            return Task::new_permission_error("You are not allowed to disconnect this user");
        }
        Task::new_empty()
    }
}

#[async_trait]
impl GenericTask for GetUserTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(user) = controller.users.get_user(&self.0) else {
            return Task::new_err(Status::not_found("User not found"));
        };

        let (server, group) = if let CurrentServer::Connected(server) = user.server() {
            let server = server.uuid();
            (
                Some(server.to_string()),
                controller
                    .servers
                    .get_server(server)
                    .and_then(|server| server.group().clone()),
            )
        } else {
            (None, None)
        };
        Task::new_ok(Item {
            name: user.id().name().clone(),
            id: user.id().uuid().to_string(),
            group,
            server,
        })
    }
}

#[async_trait]
impl GenericTask for GetUserFromNameTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(user) = controller.users.get_user_from_name(&self.0) else {
            return Task::new_err(Status::not_found("User not found"));
        };

        let (server, group) = if let CurrentServer::Connected(server) = user.server() {
            let server = server.uuid();
            (
                Some(server.to_string()),
                controller
                    .servers
                    .get_server(server)
                    .and_then(|server| server.group().clone()),
            )
        } else {
            (None, None)
        };
        Task::new_ok(Item {
            name: user.id().name().clone(),
            id: user.id().uuid().to_string(),
            group,
            server,
        })
    }
}

// TODO: This call is very expensive
// TODO: Remove or find a different solution
#[async_trait]
impl GenericTask for GetUsersTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            users: controller
                .users
                .get_users()
                .iter()
                .map(|user| {
                    let (server, group) = if let CurrentServer::Connected(server) = user.server() {
                        let server = server.uuid();
                        (
                            Some(server.to_string()),
                            controller
                                .servers
                                .get_server(server)
                                .and_then(|server| server.group().clone()),
                        )
                    } else {
                        (None, None)
                    };
                    Item {
                        name: user.id().name().clone(),
                        id: user.id().uuid().to_string(),
                        group,
                        server,
                    }
                })
                .collect(),
        })
    }
}

#[async_trait]
impl GenericTask for UserCountTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(controller.users.get_user_count())
    }
}
