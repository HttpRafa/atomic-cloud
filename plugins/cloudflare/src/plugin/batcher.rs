use std::collections::{hash_map::Drain, HashMap};

use super::dns::Record;

pub enum Action {
    Create(Record),
    Delete(String), // UUID of server
}

#[derive(Default)]
pub struct Batcher {
    inner: HashMap<String, Action>,
}

impl Batcher {
    pub fn create(&mut self, record: Record) {
        self.inner
            .insert(record.uuid.clone(), Action::Create(record));
    }
    pub fn delete(&mut self, uuid: String) {
        self.inner.insert(uuid.clone(), Action::Delete(uuid));
    }
    pub fn drain(&mut self) -> Drain<String, Action> {
        self.inner.drain()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
