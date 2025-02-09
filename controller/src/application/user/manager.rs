use std::collections::HashMap;

use anyhow::Result;
use simplelog::{info, warn};
use uuid::Uuid;

use crate::{
    application::{
        auth::ActionResult,
        server::{NameAndUuid, Server},
    },
    config::Config,
};

use super::{CurrentServer, User};

pub struct UserManager {
    users: HashMap<Uuid, User>,
}

impl UserManager {
    pub fn init() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn remove_users_on_server(&mut self, server: &Uuid) -> u32 {
        let mut amount = 0;
        self.users.retain(|_, user| {
            if let CurrentServer::Connected(current) = &user.server {
                if current.uuid() == server {
                    info!(
                        "User {}[{}] disconnected from server {}",
                        user.id,
                        user.id.uuid().to_string(),
                        current.name(),
                    );
                    amount += 1;
                    return false;
                }
            }
            true
        });
        amount
    }

    pub fn user_connected(&mut self, server: &mut Server, id: NameAndUuid) {
        // Update server user count
        server.set_connected_users(server.connected_users() + 1);

        // Update internal user list
        if let Some(user) = self.users.get_mut(id.uuid()) {
            match &user.server {
                CurrentServer::Connected(_) => {
                    warn!(
                        "User {}[{}] was never flagged as transferring but switched to server {}",
                        id,
                        id.uuid().to_string(),
                        server.id(),
                    );
                }
                CurrentServer::Transfering(_) => {
                    info!(
                        "User {}[{}] successfully transferred to server {}",
                        id,
                        id.uuid().to_string(),
                        server.id(),
                    );
                }
            }
            user.server = CurrentServer::Connected(server.id().clone());
        } else {
            info!(
                "User {}[{}] connected to server {}",
                id,
                id.uuid().to_string(),
                server.id()
            );
            self.users.insert(
                *id.uuid(),
                User {
                    id,
                    server: CurrentServer::Connected(server.id().clone()),
                },
            );
        }
    }

    pub fn user_disconnected(&mut self, server: &mut Server, uuid: &Uuid) -> ActionResult {
        // Update server user count
        server.set_connected_users(server.connected_users() - 1);

        // Update internal user list
        if let Some(user) = self.users.get(uuid) {
            if let CurrentServer::Connected(current) = &user.server {
                // Verify that the user is connected to the server
                if current.uuid() == server.id().uuid() {
                    info!(
                        "User {}[{}] disconnected from server {}",
                        user.id,
                        user.id.uuid().to_string(),
                        server.id(),
                    );
                    self.users.remove(uuid);
                } else {
                    return ActionResult::Denied;
                }
            }
        }
        ActionResult::Allowed
    }

    pub fn get_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    pub fn get_user(&self, uuid: &Uuid) -> Option<&User> {
        self.users.get(uuid)
    }
    pub fn get_user_mut(&mut self, uuid: &Uuid) -> Option<&mut User> {
        self.users.get_mut(uuid)
    }
}

// Ticking
impl UserManager {
    #[allow(clippy::unnecessary_wraps)]
    pub fn tick(&mut self, config: &Config) -> Result<()> {
        self.users.retain(|_, user| {
            if let CurrentServer::Transfering((timestamp, to)) = &user.server {
                if timestamp.elapsed() >= *config.transfer_timeout() {
                    warn!(
                        "User {}[{}] transfer to server {} timed out",
                        user.id,
                        user.id.uuid(),
                        to,
                    );
                    return false;
                }
            }
            true
        });
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
