use serde::{Deserialize, Serialize};

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

#[derive(Deserialize, Serialize)]
pub struct Backend {
    url: String,
    token: String
}

impl Backend {
    pub fn _node_exists(&self, _name: &str) -> bool {
        true
    }
}

impl SaveToTomlFile for Backend {}
impl LoadFromTomlFile for Backend {}