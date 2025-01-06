use std::collections::HashMap;

use simplelog::{error, info};
use uuid::Uuid;

use super::network::CloudConnectionHandle;

pub struct Users {
    /* Users */
    users: HashMap<String, User>,

    /* Network */
    connection: CloudConnectionHandle,
}

impl Users {
    pub fn new(connection: CloudConnectionHandle) -> Self {
        Self {
            users: HashMap::new(),
            connection,
        }
    }

    pub async fn handle_connect(&mut self, name: String, uuid: Uuid) {
        info!("<blue>{}</> connected to unit", name);

        if let Err(error) = self
            .connection
            .user_connected(name.clone(), uuid.to_string())
            .await
        {
            error!(
                "<red>Failed</> to notify controller that user connected: {}",
                error
            );
        }
        self.users.insert(name.clone(), User { name, uuid });
    }

    pub async fn handle_disconnect(&mut self, name: String) {
        if let Some(user) = self.users.remove(&name) {
            info!("<blue>{}</> disconnected from unit", user.name);

            if let Err(error) = self
                .connection
                .user_disconnected(user.uuid.to_string())
                .await
            {
                error!(
                    "<red>Failed</> to notify controller that user connected: {}",
                    error
                );
            }
        }
    }

    pub async fn get_user_from_uuid(&self, uuid: Uuid) -> Option<&User> {
        self.users.values().find(|user| user.uuid == uuid)
    }
}

pub struct User {
    pub name: String,
    pub uuid: Uuid,
}
