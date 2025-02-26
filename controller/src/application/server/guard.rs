use std::sync::{Arc, Weak};

pub struct Guard(#[allow(dead_code)] Arc<()>);
pub struct WeakGuard(Weak<()>);

impl Guard {
    pub fn new() -> (Guard, WeakGuard) {
        let arc = Arc::new(());
        let weak = Arc::downgrade(&arc);
        (Guard(arc), WeakGuard(weak))
    }
}

impl WeakGuard {
    pub fn is_dropped(&self) -> bool {
        self.0.strong_count() == 0
    }
}
