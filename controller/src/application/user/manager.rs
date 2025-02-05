use std::collections::HashMap;

use anyhow::Result;
use simplelog::{info, warn};
use uuid::Uuid;

use crate::application::server::Server;

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
                        user.name,
                        user.uuid.to_string(),
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

    pub fn user_connected(&mut self, server: &mut Server, name: String, uuid: Uuid) {
        // Update server user count
        server.set_connected_users(server.connected_users() + 1);

        // Update internal user list
        if let Some(user) = self.users.get_mut(&uuid) {
            match &user.server {
                CurrentServer::Connected(_) => {
                    warn!(
                        "User {}[{}] was never flagged as transferring but switched to server {}",
                        name,
                        uuid.to_string(),
                        server.id(),
                    );
                }
                CurrentServer::Transfering(_) => {
                    info!(
                        "User {}[{}] successfully transferred to server {}",
                        name,
                        uuid.to_string(),
                        server.id(),
                    );
                }
            }
            user.server = CurrentServer::Connected(server.id().clone());
        } else {
            info!("User {}[{}] connected to server {}", name, uuid.to_string(), server.id());
            self.users.insert(uuid, User {
                name,
                uuid,
                server: CurrentServer::Connected(server.id().clone()),
            });
        }
    }

    pub fn user_disconnected(&mut self, server: &mut Server, uuid: &Uuid) {
        // Update server user count
        server.set_connected_users(server.connected_users() - 1);

        // Update internal user list
        if let Some(user) = self.users.get(uuid) {
            if let CurrentServer::Connected(current) = &user.server {
                // Verify that the user is connected to the server
                if current.uuid() == server.id().uuid() {
                    info!(
                        "User {}[{}] disconnected from server {}",
                        user.name,
                        user.uuid.to_string(),
                        server.id(),
                    );
                    self.users.remove(uuid);
                }
            }
        }
    }
}

// Ticking
impl UserManager {
    pub async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
