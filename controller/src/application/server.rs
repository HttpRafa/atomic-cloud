use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    time::Duration,
};

use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;
use uuid::Uuid;

use crate::network::client::TransferMsg;

use super::node::Allocation;

pub mod manager;
pub mod screen;

#[derive(Getters, Setters, MutGetters)]
pub struct Server {
    /* Settings */
    #[getset(get = "pub")]
    id: NameAndUuid,
    #[getset(get = "pub")]
    group: Option<String>,
    #[getset(get = "pub")]
    node: String,
    #[getset(get = "pub")]
    allocation: Allocation,
    #[getset(get = "pub")]
    token: String,

    /* Users */
    #[getset(get = "pub", set = "pub")]
    connected_users: u32,

    /* States */
    #[getset(get = "pub", get_mut = "pub")]
    heart: Heart,
    #[getset(get = "pub", get_mut = "pub")]
    flags: Flags,
    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    state: State,
    #[getset(get = "pub", set = "pub")]
    ready: bool,
}

#[derive(Clone, Getters, MutGetters)]
pub struct NameAndUuid {
    #[getset(get = "pub", get_mut = "pub")]
    name: String,
    #[getset(get = "pub", get_mut = "pub")]
    uuid: Uuid,
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

#[derive(PartialEq, Clone)]
pub enum State {
    Starting,
    Restarting,
    Running,
    Stopping,
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

pub struct Heart {
    next_beat: Instant,
    timeout: Duration,
}

impl Server {
    pub fn new_transfer(&self, user: &Uuid) -> Option<TransferMsg> {
        let port = self.allocation.primary_port()?;
        Some(TransferMsg {
            id: user.to_string(),
            host: port.host.clone(),
            port: u32::from(port.port),
        })
    }
}

#[derive(Default)]
pub struct Flags {
    /* Required for the group system */
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

impl Heart {
    pub fn new(startup_time: Duration, timeout: Duration) -> Self {
        Self {
            next_beat: Instant::now() + startup_time,
            timeout,
        }
    }
    pub fn beat(&mut self) {
        self.next_beat = Instant::now() + self.timeout;
    }
    pub fn is_dead(&self) -> bool {
        Instant::now() > self.next_beat
    }
}

impl NameAndUuid {
    pub fn generate(name: String) -> Self {
        Self {
            name,
            uuid: Uuid::new_v4(),
        }
    }
    pub fn new(name: String, uuid: Uuid) -> Self {
        Self { name, uuid }
    }
}

impl Resources {
    pub fn new(memory: u32, swap: u32, cpu: u32, io: u32, disk: u32, ports: u32) -> Self {
        Self {
            memory,
            swap,
            cpu,
            io,
            disk,
            ports,
        }
    }
}

impl FallbackPolicy {
    pub fn new(enabled: bool, priority: i32) -> Self {
        Self { enabled, priority }
    }
}

impl Spec {
    pub fn new(
        settings: HashMap<String, String>,
        environment: HashMap<String, String>,
        disk_retention: DiskRetention,
        image: String,
        max_players: u32,
        fallback: FallbackPolicy,
    ) -> Self {
        Self {
            settings,
            environment,
            disk_retention,
            image,
            max_players,
            fallback,
        }
    }
}

impl Display for NameAndUuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
