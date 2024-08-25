use std::{ops::Deref, sync::Arc};

use log::error;
use log::info;
use log::warn;

use crate::controller::{
    event::{transfer::UserTransferRequested, EventKey},
    group::WeakGroupHandle,
    server::{ServerHandle, WeakServerHandle},
};

use super::{CurrentServer, UserHandle, Users, WeakUserHandle};

impl Users {
    pub fn transfer_all_users(&self, _server: &ServerHandle) -> u32 {
        // TODO: Move all players from server to another server
        0
    }

    pub fn resolve_transfer(&self, user: &UserHandle, target: &TransferTarget) -> Option<Transfer> {
        if let CurrentServer::Connected(from) = user.server.lock().unwrap().deref() {
            match target {
                TransferTarget::Server(to) => {
                    return Some(Transfer::new(
                        Arc::downgrade(user),
                        from.clone(),
                        to.clone(),
                    ));
                }
                TransferTarget::Group(group) => {
                    if let Some(group) = group.upgrade() {
                        if let Some(to) = group.get_free_server() {
                            return Some(Transfer::new(
                                Arc::downgrade(user),
                                from.clone(),
                                Arc::downgrade(&to),
                            ));
                        } else {
                            warn!("Failed to find free server in group {} while resolving transfer of user {}", group.name, user.name);
                        }
                    } else {
                        error!(
                            "Failed to upgrade group handle while resolving transfer of user {}",
                            user.name
                        );
                    }
                }
            }
        }

        None
    }

    pub fn transfer_user(&self, transfer: Transfer) -> bool {
        if let Some((user, from, to)) = transfer.get_strong() {
            info!(
                "Transfering user {} from {} to server {}",
                user.name, from.name, to.name
            );

            let controller = self
                .controller
                .upgrade()
                .expect("Failed to upgrade controller");
            controller.get_event_bus().dispatch(
                &EventKey::Transfer(from.uuid),
                &UserTransferRequested {
                    transfer: transfer.clone(),
                },
            );

            *user.server.lock().unwrap() = CurrentServer::Transfering(transfer);
            return true;
        } else {
            error!("Failed to transfer user because some required information is missing");
        }

        false
    }
}

pub enum TransferTarget {
    Server(WeakServerHandle),
    Group(WeakGroupHandle),
}

#[derive(Clone, Debug)]
pub struct Transfer {
    pub user: WeakUserHandle,
    pub from: WeakServerHandle,
    pub to: WeakServerHandle,
}

impl Transfer {
    pub fn new(user: WeakUserHandle, from: WeakServerHandle, to: WeakServerHandle) -> Self {
        Self { user, from, to }
    }

    pub fn get_strong(&self) -> Option<(UserHandle, ServerHandle, ServerHandle)> {
        let user = self.user.upgrade()?;
        let from = self.from.upgrade()?;
        let to = self.to.upgrade()?;
        Some((user, from, to))
    }
}
