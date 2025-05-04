use crate::generated::plugin::system::{data_types::Server, server::get_server};

use super::{config::Weight, math::WeightCalc};

pub mod manager;

pub struct Record {
    pub server: Server,
    pub id: String,
    pub weight: u16,
}

impl Record {
    pub fn new(values: &Weight, server: Server, id: String) -> Self {
        let mut record = Self {
            server,
            id,
            weight: 0,
        };
        record.update_weight(values);
        record
    }

    fn update_weight(&mut self, values: &Weight) -> bool {
        let new = WeightCalc::calc_from(self.server.connected_users, values);
        let changed = new != self.weight;
        self.weight = new;
        changed
    }

    pub fn update(&mut self, values: &Weight) -> bool {
        if let Some(server) = get_server(&self.server.uuid) {
            self.server = server;
        }

        self.update_weight(values)
    }
}
