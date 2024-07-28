use std::sync::{Arc, Mutex, Weak};

use uuid::Uuid;

use super::{server::ServerHandle, WeakControllerHandle};

pub type UserHandle = Arc<User>;
pub type WeakUserHandle = Weak<User>;

pub struct Users {
    controller: WeakControllerHandle,

    /* Users that joined some started server */
    users: Mutex<Vec<UserHandle>>,
}

impl Users {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            users: Mutex::new(Vec::new()),
        }
    }

    pub fn tick(&self) {}

    pub fn user_joined(&self, _server: ServerHandle, _name: String, _uuid: Uuid) {}

    pub fn user_left(&self, _server: ServerHandle, _uuid: Uuid) {}
}

pub struct User {
    pub name: String,
    pub uuid: Uuid,
}
