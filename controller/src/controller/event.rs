use channel::ChannelMessageSended;
use colored::Colorize;
use log::debug;

use super::server::{ServerHandle, WeakServerHandle};
use crate::network::server::proto::ChannelMessage;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};

pub mod channel;

#[derive(Eq, PartialEq)]
pub enum EventKey {
    Channel(String),
    Custom(TypeId),
}

pub trait Event: Any + Send + Sync + Debug {}

pub type EventListener<E> = Box<dyn Fn(&E) + Send + Sync>;

struct RegisteredListener {
    server: Option<WeakServerHandle>,
    listener: Box<dyn Any + Send + Sync>,
}

pub struct EventBus {
    listeners: Mutex<HashMap<EventKey, Vec<RegisteredListener>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_listener<E: Event>(&self, key: EventKey, listener: EventListener<E>) {
        let registered_listener = RegisteredListener {
            server: None,
            listener: Box::new(listener),
        };
        self.listeners
            .lock()
            .unwrap()
            .entry(key)
            .or_default()
            .push(registered_listener);
    }

    pub fn register_listener_with_server<E: Event>(
        &self,
        key: EventKey,
        server: WeakServerHandle,
        listener: EventListener<E>,
    ) {
        let registered_listener = RegisteredListener {
            server: Some(server),
            listener: Box::new(listener),
        };
        self.listeners
            .lock()
            .unwrap()
            .entry(key)
            .or_default()
            .push(registered_listener);
    }

    pub fn unregister_listener(&self, key: EventKey, server: &ServerHandle) {
        let mut listeners = self.listeners.lock().unwrap();
        if let Some(registered_listeners) = listeners.get_mut(&key) {
            registered_listeners.retain(|registered_listener| {
                if let Some(weak_server) = &registered_listener.server {
                    if let Some(strong_server) = weak_server.upgrade() {
                        if Arc::ptr_eq(server, &strong_server) {
                            return false;
                        }
                    } else {
                        return false; // Server is dead
                    }
                }
                true
            });
        }
    }

    pub fn cleanup_server(&self, server: &ServerHandle) {
        let mut listeners = self.listeners.lock().unwrap();
        for (_, registered_listeners) in listeners.iter_mut() {
            registered_listeners.retain(|registered_listener| {
                if let Some(weak_server) = &registered_listener.server {
                    if let Some(strong_server) = weak_server.upgrade() {
                        if Arc::ptr_eq(server, &strong_server) {
                            return false;
                        }
                    } else {
                        return false; // Server is dead
                    }
                }
                true
            });
        }
    }

    pub fn dispatch<E: Event>(&self, event: &E) -> u32 {
        debug!("[{}] Dispatching event: {:?}", "EVENTS".blue(), event);

        let mut count = 0;
        let listeners = self.listeners.lock().unwrap();
        if let Some(registered_listeners) = listeners.get(&EventKey::Custom(TypeId::of::<E>())) {
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

    pub fn dispatch_channel_message(&self, message: ChannelMessage) -> u32 {
        debug!(
            "[{}] Dispatching channel message: {:?}",
            "EVENTS".blue(),
            message
        );

        let mut count = 0;
        let listeners = self.listeners.lock().unwrap();
        if let Some(registered_listeners) =
            listeners.get(&EventKey::Channel(message.channel.clone()))
        {
            let event = ChannelMessageSended { message };
            for registered_listener in registered_listeners {
                if let Some(listener) = registered_listener
                    .listener
                    .downcast_ref::<EventListener<ChannelMessageSended>>()
                {
                    listener(&event);
                    count += 1;
                }
            }
        }
        count
    }
}

impl Hash for EventKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            EventKey::Channel(channel) => {
                state.write_u8(0);
                channel.hash(state);
            }
            EventKey::Custom(type_id) => {
                state.write_u8(1);
                type_id.hash(state);
            }
        }
    }
}
