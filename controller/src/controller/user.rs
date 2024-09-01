use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex, Weak},
    time::Instant,
};

use colored::Colorize;
use log::{debug, info, warn};
use transfer::Transfer;
use uuid::Uuid;

use super::{
    server::{ServerHandle, WeakServerHandle},
    WeakControllerHandle,
};

pub mod transfer;

pub type UserHandle = Arc<User>;
pub type WeakUserHandle = Weak<User>;

pub struct Users {
    controller: WeakControllerHandle,

    /* Users that joined some started server */
    users: Mutex<HashMap<Uuid, UserHandle>>,
}

impl Users {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            users: Mutex::new(HashMap::new()),
        }
    }

    pub fn tick(&self) {
        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        let mut users = self.users.lock().unwrap();
        users.retain(|_, user| {
            if let CurrentServer::Transfering(transfer) = user.server.lock().unwrap().deref() {
                if Instant::now().duration_since(transfer.timestamp)
                    > controller.configuration.timings.transfer.unwrap()
                {
                    if let Some(to) = transfer.to.upgrade() {
                        warn!(
                            "User {}[{}] failed to transfer to server {} in time",
                            user.name.blue(),
                            user.uuid.to_string().blue(),
                            to.name.blue()
                        );
                    }
                    return false;
                }
            }
            true
        });
    }

    pub fn handle_user_connected(&self, server: ServerHandle, name: String, uuid: Uuid) {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.get(&uuid) {
            let mut current_server = user.server.lock().unwrap();
            match current_server.deref() {
                CurrentServer::Connected(_) => {
                    *current_server = CurrentServer::Connected(Arc::downgrade(&server));
                    warn!(
                        "User {}[{}] was never flagged as transferring but switched to server {}",
                        name.blue(),
                        uuid.to_string().blue(),
                        server.name.blue()
                    );
                }
                CurrentServer::Transfering(_) => {
                    *current_server = CurrentServer::Connected(Arc::downgrade(&server));
                    info!(
                        "User {}[{}] successfully transferred to server {}",
                        name.blue(),
                        uuid.to_string().blue(),
                        server.name.blue()
                    );
                }
            }
        } else {
            info!(
                "User {}[{}] {} to server {}",
                name.blue(),
                uuid.to_string().blue(),
                "connected".green(),
                server.name.blue()
            );
            users.insert(uuid, self.create_user(name, uuid, &server));
        }
    }

    pub fn handle_user_disconnected(&self, server: ServerHandle, uuid: Uuid) {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.get(&uuid).cloned() {
            if let CurrentServer::Connected(weak_server) = user.server.lock().unwrap().deref() {
                if let Some(strong_server) = weak_server.upgrade() {
                    // Verify if the user is connected to the server that is saying he is disconnecting
                    if Arc::ptr_eq(&strong_server, &server) {
                        info!(
                            "User {}[{}] {} from server {}",
                            user.name.blue(),
                            user.uuid.to_string().blue(),
                            "disconnect".red(),
                            strong_server.name.blue(),
                        );
                        users.remove(&user.uuid);
                    }
                }
            }
        }
    }

    pub fn cleanup_users(&self, dead_server: &ServerHandle) -> u32 {
        let mut amount = 0;
        self.users.lock().unwrap().retain(|_, user| {
            if let CurrentServer::Connected(weak_server) = user.server.lock().unwrap().deref() {
                if let Some(server) = weak_server.upgrade() {
                    if Arc::ptr_eq(&server, dead_server) {
                        info!(
                            "User {}[{}] {} from server {}",
                            user.name.blue(),
                            user.uuid.to_string().blue(),
                            "disconnect".red(),
                            server.name.blue(),
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

    pub fn get_users_on_server(&self, server: &ServerHandle) -> Vec<UserHandle> {
        self.users
            .lock()
            .unwrap()
            .values()
            .filter(|user| {
                if let CurrentServer::Connected(weak_server) = user.server.lock().unwrap().deref() {
                    if let Some(strong_server) = weak_server.upgrade() {
                        return Arc::ptr_eq(&strong_server, server);
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    pub fn get_user(&self, uuid: Uuid) -> Option<UserHandle> {
        self.users.lock().unwrap().get(&uuid).cloned()
    }

    fn create_user(&self, name: String, uuid: Uuid, server: &ServerHandle) -> UserHandle {
        Arc::new(User {
            name,
            uuid,
            server: Mutex::new(CurrentServer::Connected(Arc::downgrade(server))),
        })
    }
}

pub enum CurrentServer {
    Connected(WeakServerHandle),
    Transfering(Transfer),
}

pub struct User {
    pub name: String,
    pub uuid: Uuid,
    pub server: Mutex<CurrentServer>,
}
