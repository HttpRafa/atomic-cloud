use crate::controller::{
    group::WeakGroupHandle,
    server::{ServerHandle, WeakServerHandle},
};

use super::{UserHandle, Users, WeakUserHandle};

impl Users {
    pub fn transfer_all_users(&self, _server: &ServerHandle) -> u32 {
        // TODO: Move all players from server to another server
        0
    }

    pub fn transfer_user(&self, _user: &UserHandle, _target: &TransferTarget) -> bool {
        // TODO: Move all players from server to another server
        false
    }
}

pub enum TransferTarget {
    Server(WeakServerHandle),
    Group(WeakGroupHandle),
}

pub struct Transfer {
    pub user: WeakUserHandle,
    pub from: WeakServerHandle,
    pub to: WeakServerHandle,
}
