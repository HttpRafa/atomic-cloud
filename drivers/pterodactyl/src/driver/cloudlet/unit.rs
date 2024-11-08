use std::time::{SystemTime, UNIX_EPOCH};

use crate::exports::cloudlet::driver::bridge::Retention;

#[derive(Clone)]
pub struct PanelUnit {
    pub id: u32,
    pub identifier: String,
    pub name: UnitName,
}

impl PanelUnit {
    pub fn new(id: u32, identifier: String, name: UnitName) -> PanelUnit {
        Self {
            id,
            identifier,
            name,
        }
    }
}

#[derive(Clone)]
pub struct UnitName {
    pub cloud_identifier: String,
    pub name: String,
    pub timestamp: u64,
    pub retention: Retention,
}

impl UnitName {
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
