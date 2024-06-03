use serde::{Deserialize, Serialize};

pub struct Servers {

}

impl Servers {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ServerResources {
    pub memory: u32,
    pub cpu: u32,
    pub disk: u32,
}