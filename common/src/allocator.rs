use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
    ops::{AddAssign, Range},
};

pub struct NumberAllocator<T> {
    next: T,
    max: T,
    available: BTreeSet<T>,
    active: HashSet<T>,
}

impl<T> NumberAllocator<T>
where
    T: Copy + Ord + Hash + AddAssign + From<u8>,
{
    pub fn new(range: Range<T>) -> Self {
        Self {
            next: range.start,
            max: range.end,
            available: BTreeSet::new(),
            active: HashSet::new(),
        }
    }

    pub fn allocate(&mut self) -> Option<T> {
        if let Some(&id) = self.available.iter().next() {
            self.available.remove(&id);
            self.active.insert(id);
            Some(id)
        } else if self.next < self.max {
            let id = self.next;
            self.next += T::from(1);
            self.active.insert(id);
            Some(id)
        } else {
            None
        }
    }

    pub fn release(&mut self, value: T) {
        if self.active.remove(&value) {
            self.available.insert(value);
        }
    }
}
