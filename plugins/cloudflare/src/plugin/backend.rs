use super::config::Config;

pub mod batch;
mod common;

#[derive(Default)]
pub struct Backend {
    token: String,
}

impl Backend {
    pub fn new(config: &Config) -> Self {
        Self {
            token: config.account.token.clone(),
        }
    }
}
