use std::collections::HashMap;

use anyhow::Result;
use simplelog::info;
use uuid::Uuid;

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
