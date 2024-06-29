use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::exports::node::driver::bridge::Resources;

/* Create Server on panel */
#[derive(Serialize, Clone)]
pub struct BCServer {
    pub name: String,
    pub node: u32,
    pub user: u32,
    pub egg: u32,
    pub docker_image: String,
    pub startup: String,
    pub environment: HashMap<String, String>,
    pub limits: BServerLimits,
    pub feature_limits: BServerFeatureLimits,
    pub allocation: BCServerAllocation,
}

#[derive(Serialize, Clone)]
pub struct BCServerAllocation {
    pub default: u32,
}

#[derive(Deserialize, Clone)]
pub struct BServer {
    pub id: u32,
    pub name: String,
}

/* Generic Data of Server */
#[derive(Deserialize, Serialize, Clone)]
pub struct BServerLimits {
    pub memory: u32,
    pub swap: u32,
    pub disk: u32,
    pub io: u32,
    pub cpu: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BServerFeatureLimits {
    pub databases: u32,
    pub backups: u32,
}

impl From<Resources> for BServerLimits {
    fn from(resources: Resources) -> Self {
        Self {
            memory: resources.memory,
            swap: resources.swap,
            disk: resources.disk,
            io: resources.io,
            cpu: resources.cpu,
        }
    }
}
