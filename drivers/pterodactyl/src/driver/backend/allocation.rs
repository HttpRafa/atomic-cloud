use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct BAllocation {
    pub id: u32,
    pub ip: String,
    pub port: u16,
    pub assigned: bool,
}
