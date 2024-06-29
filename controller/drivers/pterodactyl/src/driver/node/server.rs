use std::time::{SystemTime, UNIX_EPOCH};

use crate::exports::node::driver::bridge::Retention;

#[derive(Clone)]
pub struct PanelServer {
    pub id: u32,
    pub identifier: String,
    pub name: ServerName,
}

impl PanelServer {
    pub fn new(id: u32, identifier: String, name: ServerName) -> PanelServer {
        Self { id, identifier, name }
    }
}

#[derive(Clone)]
pub struct ServerName {
    pub cloud_identifier: String,
    pub name: String,
    pub timestamp: u64,
    pub retention: Retention,
}

impl ServerName {
    pub fn new(cloud_identifier: &str, name: &str, retention: &Retention) -> Self {
        Self {
            cloud_identifier: cloud_identifier.to_owned(),
            name: name.to_owned(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            retention: retention.to_owned(),
        }
    }

    pub fn generate(&self) -> String {
        if self.retention == Retention::Permanent {
            return format!("{}@{}", self.name, self.cloud_identifier);
        }
        format!("{}@{}#{}", self.name, self.cloud_identifier, self.timestamp)
    }
}
