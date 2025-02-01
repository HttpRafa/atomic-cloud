use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod manager;

pub struct Server {
    /* Settings */
    name: String,
    uuid: Uuid,
    group: Option<String>,
    node: String,

    /* Users */
    connected_users: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Resources {
    memory: u32,
    swap: u32,
    cpu: u32,
    io: u32,
    disk: u32,
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

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FallbackPolicy {
    enabled: bool,
    priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Spec {
    settings: HashMap<String, String>,
    environment: HashMap<String, String>,
    disk_retention: DiskRetention,
    image: String,

    max_players: u32,
    fallback: FallbackPolicy,
}

impl Server {
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_uuid(&self) -> &Uuid {
        &self.uuid
    }
    pub fn get_group(&self) -> &Option<String> {
        &self.group
    }
    pub fn get_node(&self) -> &String {
        &self.node
    }
    pub fn get_connected_users(&self) -> u32 {
        self.connected_users
    }
}

impl Resources {
    pub fn get_memory(&self) -> u32 {
        self.memory
    }
    pub fn get_swap(&self) -> u32 {
        self.swap
    }
    pub fn get_cpu(&self) -> u32 {
        self.cpu
    }
    pub fn get_io(&self) -> u32 {
        self.io
    }
    pub fn get_disk(&self) -> u32 {
        self.disk
    }
    pub fn get_ports(&self) -> u32 {
        self.ports
    }
}

impl Spec {
    pub fn get_settings(&self) -> &HashMap<String, String> {
        &self.settings
    }
    pub fn get_environment(&self) -> &HashMap<String, String> {
        &self.environment
    }
    pub fn get_disk_retention(&self) -> &DiskRetention {
        &self.disk_retention
    }
    pub fn get_image(&self) -> &String {
        &self.image
    }
    pub fn get_max_players(&self) -> u32 {
        self.max_players
    }
    pub fn get_fallback(&self) -> &FallbackPolicy {
        &self.fallback
    }
}
