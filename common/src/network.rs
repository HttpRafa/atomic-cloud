use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HostAndPort<S = String> {
    pub host: S,
    pub port: u16,
}

impl HostAndPort {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

impl Display for HostAndPort {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}:{}", self.host, self.port)
    }
}
