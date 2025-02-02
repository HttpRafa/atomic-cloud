use std::{collections::HashMap, time::Duration};

use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use uuid::Uuid;

pub mod manager;

#[derive(Getters, MutGetters)]
pub struct Server {
    /* Settings */
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    uuid: Uuid,
    #[getset(get = "pub")]
    group: Option<String>,
    #[getset(get = "pub")]
    node: String,

    /* Users */
    #[getset(get = "pub")]
    connected_users: u32,

    /* States */
    #[getset(get = "pub", get_mut = "pub")]
    health: Health,
    #[getset(get = "pub", get_mut = "pub")]
    flags: Flags,
}

#[derive(Serialize, Deserialize, Clone, Default, Getters)]
pub struct Resources {
    #[getset(get = "pub")]
    memory: u32,
    #[getset(get = "pub")]
    swap: u32,
    #[getset(get = "pub")]
    cpu: u32,
    #[getset(get = "pub")]
    io: u32,
    #[getset(get = "pub")]
    disk: u32,
    #[getset(get = "pub")]
    ports: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum DiskRetention {
    #[serde(rename = "temporary")]
    #[default]
    Temporary,
    #[serde(rename = "permanent")]
    Permanent,
}

#[derive(Serialize, Deserialize, Clone, Default, Getters)]
pub struct FallbackPolicy {
    #[getset(get = "pub")]
    enabled: bool,
    #[getset(get = "pub")]
    priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Default, Getters)]
pub struct Spec {
    #[getset(get = "pub")]
    settings: HashMap<String, String>,
    #[getset(get = "pub")]
    environment: HashMap<String, String>,
    #[getset(get = "pub")]
    disk_retention: DiskRetention,
    #[getset(get = "pub")]
    image: String,

    #[getset(get = "pub")]
    max_players: u32,
    #[getset(get = "pub")]
    fallback: FallbackPolicy,
}

pub struct Health {
    next_check: Instant,
    timeout: Duration,
}

pub struct Flags {
    /* Required for the deployment system */
    pub stop: Option<Instant>,
}

impl Flags {
    pub fn is_stop_set(&self) -> bool {
        self.stop.is_some()
    }
    pub fn should_stop(&self) -> bool {
        self.stop.is_some_and(|stop| stop < Instant::now())
    }
    pub fn replace_stop(&mut self, timeout: Duration) {
        self.stop = Some(Instant::now() + timeout);
    }
    pub fn clear_stop(&mut self) {
        self.stop = None;
    }
}

impl Health {
    pub fn new(startup_time: Duration, timeout: Duration) -> Self {
        Self {
            next_check: Instant::now() + startup_time,
            timeout,
        }
    }
    pub fn reset(&mut self) {
        self.next_check = Instant::now() + self.timeout;
    }
    pub fn is_dead(&self) -> bool {
        Instant::now() > self.next_check
    }
}
