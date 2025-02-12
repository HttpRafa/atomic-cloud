use std::collections::VecDeque;

use super::ScreenMessage;

const SCREEN_CACHE_SIZE: usize = 91;

pub struct ScreenCache {
    items: VecDeque<ScreenMessage>,
}

impl ScreenCache {
    pub fn new() -> Self {
        Self {
            items: VecDeque::with_capacity(SCREEN_CACHE_SIZE),
        }
    }

    pub fn push(&mut self, item: ScreenMessage) {
        self.items.push_back(item);

        if self.items.len() > SCREEN_CACHE_SIZE {
            self.items.pop_front();
        }
    }

    pub fn get_items(&self) -> &VecDeque<ScreenMessage> {
        &self.items
    }
}
