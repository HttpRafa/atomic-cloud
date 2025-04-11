use std::collections::VecDeque;

pub struct FixedSizeCache<T> {
    items: VecDeque<T>,
    size: usize,
}

impl<T: Clone> FixedSizeCache<T> {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(size),
            size,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.items.len() == self.size {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }

    pub fn extend(&mut self, items: Vec<T>) {
        for item in items {
            self.push(item);
        }
    }

    #[must_use]
    pub fn clone_items(&self) -> Vec<T> {
        self.items.iter().cloned().collect()
    }

    #[must_use]
    pub fn has_data(&self) -> bool {
        !self.items.is_empty()
    }
}
