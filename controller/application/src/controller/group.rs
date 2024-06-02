use super::node::Nodes;

pub struct Groups {

}

impl Groups {
    pub async fn load_all(_nodes: &Nodes) -> Self {
        Self {}
    }
    pub fn tick(&self) {
        // Tick server manager
        // Check if all server have send there heartbeats etc..
    }
}