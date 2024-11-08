use colored::Colorize;
use log::debug;
use uuid::Uuid;

use super::unit::{UnitHandle, WeakUnitHandle};

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, RwLock},
};

pub mod channel;
pub mod transfer;

#[derive(Eq, PartialEq)]
pub enum EventKey {
    Channel(String),
    Transfer(Uuid),
    Custom(TypeId),
}

pub trait Event: Any + Send + Sync + Debug {}

pub type EventListener<E> = Box<dyn Fn(&E) + Send + Sync>;

struct RegisteredListener {
    unit: Option<WeakUnitHandle>,
    listener: Box<dyn Any + Send + Sync>,
}

pub struct EventBus {
    listeners: RwLock<HashMap<EventKey, Vec<RegisteredListener>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_listener<E: Event>(&self, key: EventKey, listener: EventListener<E>) {
        let registered_listener = RegisteredListener {
            unit: None,
            listener: Box::new(listener),
        };
        self.listeners
            .write()
            .unwrap()
            .entry(key)
            .or_default()
            .push(registered_listener);
    }

    pub fn register_listener_under_unit<E: Event>(
        &self,
        key: EventKey,
        unit: WeakUnitHandle,
        listener: EventListener<E>,
    ) {
        let registered_listener = RegisteredListener {
            unit: Some(unit),
            listener: Box::new(listener),
        };
        self.listeners
            .write()
            .unwrap()
            .entry(key)
            .or_default()
            .push(registered_listener);
    }

    pub fn unregister_listener(&self, key: EventKey, unit: &UnitHandle) {
        let mut listeners = self.listeners.write().unwrap();
        if let Some(registered_listeners) = listeners.get_mut(&key) {
            registered_listeners.retain(|registered_listener| {
                if let Some(weak_unit) = &registered_listener.unit {
                    if let Some(strong_unit) = weak_unit.upgrade() {
                        if Arc::ptr_eq(unit, &strong_unit) {
                            return false;
                        }
                    } else {
                        return false; // Unit is dead
                    }
                }
                true
            });
        }
    }

    pub fn cleanup_unit(&self, unit: &UnitHandle) {
        let mut listeners = self.listeners.write().unwrap();
        for (_, registered_listeners) in listeners.iter_mut() {
            registered_listeners.retain(|registered_listener| {
                if let Some(weak_unit) = &registered_listener.unit {
                    if let Some(strong_unit) = weak_unit.upgrade() {
                        if Arc::ptr_eq(unit, &strong_unit) {
                            return false;
                        }
                    } else {
                        return false; // Unit is dead
                    }
                }
                true
            });
        }
    }

    pub fn dispatch<E: Event>(&self, key: &EventKey, event: &E) -> u32 {
        debug!("[{}] Dispatching event: {:?}", "EVENTS".blue(), event);

        let mut count = 0;
        let listeners = self.listeners.read().unwrap();
        if let Some(registered_listeners) = listeners.get(key) {
            for registered_listener in registered_listeners {
                if let Some(listener) = registered_listener
                    .listener
                    .downcast_ref::<EventListener<E>>()
                {
                    listener(event);
                    count += 1;
                }
            }
        }
        count
    }

    pub fn dispatch_custom<E: Event>(&self, event: &E) -> u32 {
        self.dispatch(&EventKey::Custom(TypeId::of::<E>()), event)
    }
}

impl Hash for EventKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            EventKey::Channel(channel) => {
                state.write_u8(0);
                channel.hash(state);
            }
            EventKey::Transfer(unit) => {
                state.write_u8(1);
                unit.hash(state);
            }
            EventKey::Custom(type_id) => {
                state.write_u8(2);
                type_id.hash(state);
            }
        }
    }
}
