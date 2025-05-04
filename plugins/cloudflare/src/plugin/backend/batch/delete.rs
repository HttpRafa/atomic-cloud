use serde::Serialize;

use crate::plugin::dns::Record;

#[derive(Serialize, Clone)]
pub struct BDelete {
    id: String,
}

impl From<&Record> for BDelete {
    fn from(value: &Record) -> Self {
        Self {
            id: value.id.clone().unwrap_or_default(),
        }
    }
}
