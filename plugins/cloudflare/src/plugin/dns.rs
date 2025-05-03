use crate::generated::plugin::system::data_types::Address;

use super::{config::Entry, math::WeightCalc};

pub mod manager;

pub struct Record {
    pub uuid: String, // UUID of server
    pub id: String,   // Id of the record at Cloudflare

    pub name: String,
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

impl Record {
    pub fn new(entry: &Entry, uuid: String, connected_users: u32, address: Address) -> Self {
        Self {
            uuid,
            id: String::new(), // We get that on the creation of the record
            name: entry.name.clone(),
            priority: entry.priority,
            weight: WeightCalc::calc_from(connected_users, &entry.weight),
            port: address.port,
            target: address.host,
        }
    }
}
