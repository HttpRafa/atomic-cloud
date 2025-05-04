use serde::{Deserialize, Serialize};

use super::{
    delete::BDelete,
    record::{BRRecord, BRecord},
};

#[derive(Serialize, Default, Clone)]
pub struct BBatch {
    pub deletes: Vec<BDelete>,
    pub patches: Vec<BRecord>,
    pub posts: Vec<BRecord>,
}

#[derive(Deserialize, Clone)]
pub struct BBatchResult {
    pub deletes: Vec<BRRecord>,
    pub patches: Vec<BRRecord>,
    pub posts: Vec<BRRecord>,
}
