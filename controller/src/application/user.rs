use std::{
    collections::HashMap,
    ops::Deref,
    sync::{atomic::Ordering, Arc, RwLock, Weak},
    time::Instant,
};

use simplelog::{debug, info, warn};
use transfer::Transfer;
use uuid::Uuid;

use super::{
    unit::{UnitHandle, WeakUnitHandle},
    WeakControllerHandle,
};

pub mod transfer;

pub type UserHandle = Arc<User>;
pub type WeakUserHandle = Weak<User>;

pub struct Users {
    controller: WeakControllerHandle,

    /* Users that joined some started unit */
    users: RwLock<HashMap<Uuid, UserHandle>>,
}

impl Users {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            users: RwLock::new(HashMap::new()),
        }
    }

    pub fn tick(&self) {
        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        let mut users = self.users.write().unwrap();
        users.retain(|_, user| {
            if let CurrentUnit::Transfering(transfer) = user.unit.read().unwrap().deref() {
                if Instant::now().duration_since(transfer.timestamp)
                    > controller.configuration.timings.transfer.unwrap()
                {
                    if let Some(to) = transfer.to.upgrade() {
                        warn!(
                            "User <blue>{}</>[<blue>{}</>] failed to transfer to unit <blue>{}</> in time",
                            user.name,
                            user.uuid.to_string(),
                            to.name
                        );
                    }
                    return false;
                }
            }
            true
        });
    }

    pub fn handle_user_connected(&self, unit: UnitHandle, name: String, uuid: Uuid) {
        // Update unit that the user is connected to
        unit.connected_users.fetch_add(1, Ordering::Relaxed);

        // Update internal user list
        let mut users = self.users.write().unwrap();
        if let Some(user) = users.get(&uuid) {
            let mut current_unit = user.unit.write().unwrap();
            match current_unit.deref() {
                CurrentUnit::Connected(_) => {
                    *current_unit = CurrentUnit::Connected(Arc::downgrade(&unit));
                    warn!(
                        "User <blue>{}</>[<blue>{}</>] was never flagged as transferring but switched to unit <blue>{}</>",
                        name,
                        uuid.to_string(),
                        unit.name
                    );
                }
                CurrentUnit::Transfering(_) => {
                    *current_unit = CurrentUnit::Connected(Arc::downgrade(&unit));
                    info!(
                        "User <blue>{}</>[<blue>{}</>] successfully transferred to unit <blue>{}</>",
                        name,
                        uuid.to_string(),
                        unit.name
                    );
                }
            }
        } else {
            info!(
                "User <blue>{}</>[<blue>{}</>] <green>connected</> to unit <blue>{}</>",
                name,
                uuid.to_string(),
                unit.name
            );
            users.insert(uuid, self.create_user(name, uuid, &unit));
        }
    }

    pub fn handle_user_disconnected(&self, unit: UnitHandle, uuid: Uuid) {
        // Update unit that the user was connected to
        unit.connected_users.fetch_sub(1, Ordering::Relaxed);

        // Update internal user list
        let mut users = self.users.write().unwrap();
        if let Some(user) = users.get(&uuid).cloned() {
            if let CurrentUnit::Connected(weak_unit) = user.unit.read().unwrap().deref() {
                if let Some(strong_unit) = weak_unit.upgrade() {
                    // Verify if the user is connected to the unit that is saying he is disconnecting
                    if Arc::ptr_eq(&strong_unit, &unit) {
                        info!(
                            "User <blue>{}</>[<blue>{}</>] <red>disconnected</> from unit <blue>{}</>",
                            user.name,
                            user.uuid.to_string(),
                            strong_unit.name,
                        );
                        users.remove(&user.uuid);
                    }
                }
            }
        }
    }

    pub fn cleanup_users(&self, dead_unit: &UnitHandle) -> u32 {
        let mut amount = 0;
        self.users.write().unwrap().retain(|_, user| {
            if let CurrentUnit::Connected(weak_unit) = user.unit.read().unwrap().deref() {
                if let Some(unit) = weak_unit.upgrade() {
                    if Arc::ptr_eq(&unit, dead_unit) {
                        info!(
                            "User <blue>{}</>[<blue>{}</>] <red>disconnected</> from unit <blue>{}</>",
                            user.name,
                            user.uuid.to_string(),
                            unit.name,
                        );
                        amount += 1;
                        return false;
                    }
                } else {
                    debug!(
                        "User <blue>{}</>[<blue>{}</>] is connected to a dead unit removing him",
                        user.name,
                        user.uuid.to_string()
                    );
                    amount += 1;
                    return false;
                }
            }
            true
        });
        amount
    }

    pub fn get_users(&self) -> Vec<UserHandle> {
        self.users.read().unwrap().values().cloned().collect()
    }

    pub fn _get_users_on_unit(&self, unit: &UnitHandle) -> Vec<UserHandle> {
        self.users
            .read()
            .unwrap()
            .values()
            .filter(|user| {
                if let CurrentUnit::Connected(weak_unit) = user.unit.read().unwrap().deref() {
                    if let Some(strong_unit) = weak_unit.upgrade() {
                        return Arc::ptr_eq(&strong_unit, unit);
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    pub fn get_user(&self, uuid: Uuid) -> Option<UserHandle> {
        self.users.read().unwrap().get(&uuid).cloned()
    }

    fn create_user(&self, name: String, uuid: Uuid, unit: &UnitHandle) -> UserHandle {
        Arc::new(User {
            name,
            uuid,
            unit: RwLock::new(CurrentUnit::Connected(Arc::downgrade(unit))),
        })
    }
}

pub enum CurrentUnit {
    Connected(WeakUnitHandle),
    Transfering(Transfer),
}

pub struct User {
    pub name: String,
    pub uuid: Uuid,
    pub unit: RwLock<CurrentUnit>,
}
