use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use colored::Colorize;
use log::{debug, info};
use uuid::Uuid;

use super::{
    server::{ServerHandle, WeakServerHandle},
    WeakControllerHandle,
};

pub type UserHandle = Arc<User>;
pub type WeakUserHandle = Weak<User>;

type UsersMap = HashMap<Uuid, UserHandle>;

pub struct Users {
    controller: WeakControllerHandle,

    /* Users that joined some started server */
    users: Mutex<UsersMap>,
}

impl Users {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            users: Mutex::new(HashMap::new()),
        }
    }

    pub fn tick(&self) {}

    pub fn handle_user_connected(&self, server: ServerHandle, name: String, uuid: Uuid) {
        let mut users = self.users.lock().unwrap();
        if let Some(_user) = users.get(&uuid) {
            // TODO: Handle user transfers
        } else {
            self.register_user(&mut users, name, uuid, &server);
        }
    }

    pub fn handle_user_disconnected(&self, server: ServerHandle, uuid: Uuid) {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.get(&uuid).cloned() {
            if let CurrentServer::Connected(weak_server) = &user.server {
                if let Some(strong_server) = weak_server.upgrade() {
                    // Verify if the user is connected to the server that is saying he is disconnecting
                    if Arc::ptr_eq(&strong_server, &server) {
                        self.unregister_user(&mut users, &user);
                    }
                }
            }
        }
    }

    pub fn cleanup_users(&self, dead_server: &ServerHandle) -> u32 {
        let mut amount = 0;
        self.users.lock().unwrap().retain(|_, user| {
            if let CurrentServer::Connected(weak_server) = &user.server {
                if let Some(server) = weak_server.upgrade() {
                    if Arc::ptr_eq(&server, dead_server) {
                        info!(
                            "User {}[{}] disconnect from server",
                            user.name.blue(),
                            user.uuid.to_string().blue()
                        );
                        amount += 1;
                        return false;
                    }
                } else {
                    debug!(
                        "User {}[{}] is connected to a dead server removing him",
                        user.name.blue(),
                        user.uuid.to_string().blue()
                    );
                    amount += 1;
                    return false;
                }
            }
            true
        });
        amount
    }

    fn register_user(&self, users: &mut UsersMap, name: String, uuid: Uuid, server: &ServerHandle) -> Option<UserHandle> {
        info!(
            "User {}[{}] connect to server {}",
            name.blue(),
            uuid.to_string().blue(),
            server.name.blue()
        );

        let user = Arc::new(User {
            name,
            uuid,
            server: CurrentServer::Connected(Arc::downgrade(server)),
        });
        users.insert(uuid, user.clone());

        Some(user)
    }

    fn unregister_user(&self, users: &mut UsersMap, user: &UserHandle) {
        info!(
            "User {}[{}] disconnect from server",
            user.name.blue(),
            user.uuid.to_string().blue()
        );
        users.remove(&user.uuid);
    }
}

pub enum CurrentServer {
    Connected(WeakServerHandle),
    Transfering,
}

pub struct User {
    pub name: String,
    pub uuid: Uuid,
    pub server: CurrentServer,
}
