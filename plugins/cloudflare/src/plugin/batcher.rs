use std::collections::VecDeque;


use super::dns::{NewRecord, Record};

pub enum Action {
    Create(NewRecord),
    Delete(Record),
}

#[derive(Default)]
pub struct Batcher {
    inner: VecDeque<Action>,
}

impl Batcher {
    pub fn create(&mut self, record: NewRecord) {
        self.inner.push_front(Action::Create(record));
    }
    pub fn delete(&mut self, record: Record) {
        self.inner.push_front(Action::Delete(record));
    }
    pub fn pop(&mut self) -> Option<Action> {
        self.inner.pop_back()
    }
}
