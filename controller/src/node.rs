use std::sync::Arc;
use crate::driver::lua::LuaDriver;

pub struct Node {
    name: String,
    capabilities: Vec<Capabilities>,
    driver: Arc<LuaDriver>
}

pub enum Capabilities {
    LimitedMemory(u32),
    UnlimitedMemory
}