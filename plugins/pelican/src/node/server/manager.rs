use std::{cell::RefCell, collections::HashMap};

use crate::generated::exports::plugin::system::bridge::{Server, Uuid};

pub struct ServerManager {
    servers: HashMap<Uuid, Server>,
}

impl ServerManager {
    pub fn init() -> RefCell<Self> {
        RefCell::new(Self {
            servers: HashMap::new(),
        })
    }
}
