use std::collections::HashMap;

use anyhow::Result;

use crate::plugin::{backend::Backend, batcher::Batcher};

use super::Record;

#[derive(Default)]
pub struct Records {
    inner: HashMap<String, Record>, // UUID to record for quick lookups
}

impl Records {
    pub fn tick(&mut self, backend: &mut Backend, batcher: &mut Batcher) -> Result<()> {
        Ok(())
    }
}
