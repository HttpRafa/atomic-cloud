use serde::{Deserialize, Serialize};

use crate::plugin::{config::Entry, dns::Record};

#[derive(Serialize, Deserialize, Clone)]
pub struct BData {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

#[derive(Serialize, Clone)]
pub struct BRecord {
    pub comment: String,
    pub data: BData,
    pub name: String,
    pub proxied: bool,
    pub ttl: u16,
    pub r#type: String,
    pub id: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct BRRecord {
    pub comment: String,
    pub id: String,
    pub data: BData,
}

impl BRecord {
    pub fn new(entry: &Entry, record: &Record) -> Option<Self> {
        let address = record.server.allocation.ports.first()?;

        Some(Self {
            comment: record.server.uuid.clone(),
            data: BData {
                priority: entry.priority,
                weight: record.weight,
                port: address.port,
                target: address.host.clone(),
            },
            name: entry.name.clone(),
            proxied: false,
            ttl: entry.ttl,
            r#type: "SRV".to_string(),
            id: record.id.clone(),
        })
    }
}
