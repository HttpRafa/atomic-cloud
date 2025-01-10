use std::time::Instant;
use std::{ops::Deref, sync::Arc};

use simplelog::{error, info, warn};

use crate::application::deployment::DeploymentHandle;
use crate::application::{
    event::{transfer::UserTransferRequested, EventKey},
    unit::{UnitHandle, WeakUnitHandle},
};

use super::{CurrentUnit, UserHandle, Users, WeakUserHandle};

impl Users {
    pub fn resolve_transfer(&self, user: &UserHandle, target: &TransferTarget) -> Option<Transfer> {
        let from = {
            let unit = user.unit.read().unwrap();
            if let CurrentUnit::Connected(from) = unit.deref() {
                from.clone()
            } else {
                return None;
            }
        };

        match target {
            TransferTarget::Unit(to) => {
                return Some(Transfer::new(
                    Arc::downgrade(user),
                    from.clone(),
                    Arc::downgrade(to),
                ));
            }
            TransferTarget::Deployment(deployment) => {
                if let Some(to) = deployment.get_free_unit() {
                    return Some(Transfer::new(
                        Arc::downgrade(user),
                        from.clone(),
                        Arc::downgrade(&to),
                    ));
                } else {
                    warn!("<red>Failed</> to find free unit in deployment <blue>{}</> while resolving transfer of user <blue>{}</>", deployment.name, user.name);
                }
            }
            TransferTarget::Fallback => {
                let controller = self
                    .controller
                    .upgrade()
                    .expect("Failed to upgrade controller. This should never happen");
                if let Some(fallback) = controller
                    .get_units()
                    .find_fallback_unit(
                        &from
                            .upgrade()
                            .expect("Failed to upgrade unit. This should never happen"),
                    )
                    .map(TransferTarget::Unit)
                {
                    return self.resolve_transfer(user, &fallback);
                } else {
                    warn!("<red>Failed</> to find fallback unit while resolving transfer of user <blue>{}</>", user.name);
                }
            }
        }

        None
    }

    pub fn transfer_user(&self, transfer: Transfer) -> bool {
        if let Some((user, from, to)) = transfer.get_strong() {
            info!(
                "Transfering user <blue>{}</> from <blue>{}</> to unit <blue>{}</>",
                user.name, from.name, to.name
            );

            let controller = self
                .controller
                .upgrade()
                .expect("Failed to upgrade controller. This should never happen");
            controller.get_event_bus().dispatch(
                &EventKey::Transfer(from.uuid),
                &UserTransferRequested {
                    transfer: transfer.clone(),
                },
            );

            *user.unit.write().unwrap() = CurrentUnit::Transfering(transfer);
            return true;
        } else {
            error!("<red>Failed</> to transfer user because some required information is missing",);
        }

        false
    }
}

pub enum TransferTarget {
    Unit(UnitHandle),
    Deployment(DeploymentHandle),
    Fallback,
}

#[derive(Clone, Debug)]
pub struct Transfer {
    pub timestamp: Instant,
    pub user: WeakUserHandle,
    pub from: WeakUnitHandle,
    pub to: WeakUnitHandle,
}

impl Transfer {
    pub fn new(user: WeakUserHandle, from: WeakUnitHandle, to: WeakUnitHandle) -> Self {
        Self {
            timestamp: Instant::now(),
            user,
            from,
            to,
        }
    }

    pub fn get_strong(&self) -> Option<(UserHandle, UnitHandle, UnitHandle)> {
        let user = self.user.upgrade()?;
        let from = self.from.upgrade()?;
        let to = self.to.upgrade()?;
        Some((user, from, to))
    }
}
