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
    pub puts: Vec<()>,
}

#[derive(Deserialize, Clone)]
pub struct BBatchResult {
    pub deletes: Option<Vec<BRRecord>>,
    pub patches: Option<Vec<BRRecord>>,
    pub posts: Option<Vec<BRRecord>>,
}
