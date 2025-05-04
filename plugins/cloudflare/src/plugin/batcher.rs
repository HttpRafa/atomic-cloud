use std::collections::HashMap;

use crate::generated::plugin::system::data_types::{Server, Uuid};

use super::config::Entry;

pub enum Action {
    Create(Server),
    Delete,
}

#[derive(Default)]
pub struct Batcher {
    inner: HashMap<String, (Entry, HashMap<Uuid, Action>)>, // Grouped by zone, entry and uuid of server
}

impl Batcher {
    pub fn create(&mut self, entry: Entry, server: Server) {
        self.inner
            .entry(entry.zone.clone())
            .or_insert((entry, HashMap::new()))
            .1
            .insert(server.uuid.clone(), Action::Create(server));
    }
    pub fn delete(&mut self, entry: Entry, uuid: String) {
        self.inner
            .entry(entry.zone.clone())
            .or_insert((entry, HashMap::new()))
            .1
            .insert(uuid, Action::Delete);
    }
    pub fn drain(&mut self, zone: &str) -> Option<&mut (Entry, HashMap<Uuid, Action>)> {
        self.inner.get_mut(zone)
    }
}
