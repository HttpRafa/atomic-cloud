use super::config::Config;

pub mod batch;
mod common;

#[derive(Default)]
pub struct Backend {
    mail: String,
    token: String,
}

impl Backend {
    pub fn new(config: &Config) -> Self {
        Self {
            mail: config.account.mail.clone(),
            token: config.account.token.clone(),
        }
    }
}
