use std::sync::{Arc, Mutex};

use log::info;

use super::node::Nodes;

type GroupHandle = Arc<Mutex<Group>>;

pub struct Groups {
    groups: Vec<GroupHandle>,
}

impl Groups {
    pub async fn load_all(_nodes: &Nodes) -> Self {
        info!("Loading groups...");

        let groups = Vec::new();
        Self { groups }
    }
    pub fn tick(&self) {
        // Tick server manager
        // Check if all server have send there heartbeats etc..
    }
}

pub struct Group {

}