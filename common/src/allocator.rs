use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
    ops::{AddAssign, Range},
};

/// A generic number allocator that manages a range of values.
///
/// It sequentially allocates numbers within a given half-open range (`[start, end)`).
/// When a number is released, it is stored and later reused before new numbers are allocated.
///
/// # Type Parameters
///
/// * `T` - A numeric type that implements `Copy`, `Ord`, `Hash`, `AddAssign`, and `From<u8>`.
///
/// # Example
///
/// ```
/// use common::allocator::NumberAllocator;
///
/// // Create an allocator for numbers 1 through 10 (exclusive).
/// let mut allocator = NumberAllocator::new(1..10);
///
/// // Allocate a number.
/// let num = allocator.allocate().expect("Allocation failed");
/// println!("Allocated: {}", num);
///
/// // Release the number back to the allocator.
/// allocator.release(num);
/// ```
pub struct NumberAllocator<T> {
    next: T,
    range: Range<T>,
    available: BTreeSet<T>,
    active: HashSet<T>,
}

impl<T> NumberAllocator<T>
where
    T: Copy + Ord + Hash + AddAssign + From<u8>,
{
    /// Constructs a new `NumberAllocator` with the specified range.
    ///
    /// The allocator will provide numbers starting from `range.start` up to, but not including, `range.end`.
    pub fn new(range: Range<T>) -> Self {
        Self {
            next: range.start,
            range,
            available: BTreeSet::new(),
            active: HashSet::new(),
        }
    }

    /// Claims a specific number, marking it as active.
    ///
    /// This method allows manually marking a number as active without allocating it.
    ///
    /// # Arguments
    ///
    /// * `number` - The number to claim.
    pub fn claim(&mut self, number: T) {
        self.active.insert(number);
    }

    /// Checks if a specific number is currently claimed.
    ///
    /// # Arguments
    ///
    /// * `number` - The number to check.
    ///
    /// # Returns
    ///
    /// `true` if the number is claimed, `false` otherwise.
    pub fn is_claimed(&self, number: T) -> bool {
        self.active.contains(&number)
    }

    /// Allocates and returns a number.
    ///
    /// If there are any numbers that have been released previously, the smallest one is reused.
    /// Otherwise, the next sequential number is allocated.
    ///
    /// Returns `None` if no numbers are available (i.e. all numbers in the range are allocated).
    ///
    /// # Returns
    ///
    /// An `Option` containing the allocated number, or `None` if no numbers are available.
    pub fn allocate(&mut self) -> Option<T> {
        if let Some(&number) = self.available.iter().next() {
            self.available.remove(&number);
            self.active.insert(number);
            Some(number)
        } else if self.next < self.range.end {
            let number = self.next;
            self.next += T::from(1);
            self.active.insert(number);
            Some(number)
        } else {
            None
        }
    }

    /// Releases a previously allocated number back to the allocator.
    ///
    /// If the number was active, it is removed from the active set and added to the available pool.
    /// Released numbers are reused before new sequential numbers are allocated.
    ///
    /// # Arguments
    ///
    /// * `number` - The number to release.
    pub fn release(&mut self, number: T) {
        if self.active.remove(&number) && self.range.contains(&number) {
            self.available.insert(number);
        }
    }
}
