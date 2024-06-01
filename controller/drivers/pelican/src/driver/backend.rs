use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    config::{LoadFromTomlFile, SaveToTomlFile, CONFIG_DIRECTORY},
    error, node::driver::log::{request_user_input, Question}, warn,
};

const BACKEND_FILE: &str = "backend.toml";

#[derive(Deserialize, Serialize)]
pub struct Backend {
    url: Option<String>,
    token: Option<String>,
}

impl Backend {
    fn new_empty() -> Self {
        Self {
            url: None,
            token: None,
        }
    }

    fn load_or_empty() -> Self {
        let path = Path::new(CONFIG_DIRECTORY).join(BACKEND_FILE);
        if path.exists() {
            Self::load_from_file(&path).unwrap_or_else(|err| {
                warn!("Failed to read backend configuration from file: {}", err);
                Self::new_empty()
            })
        } else {
            Self::new_empty()
        }
    }

    pub fn new_filled() -> Option<Self> {
        let mut backend = Self::load_or_empty();

        if backend.url.is_none() {
            backend.url = request_user_input(Question::Text, "What is the url of the pelican panel?", &["http://panel.gameserver.local.example.com".to_string()]);
            if backend.url.is_none() {
                return None;
            }
        }

        if backend.token.is_none() {
            backend.token = request_user_input(Question::Password, "What is the token of the pelican panel?", &[]);
            if backend.token.is_none() {
                return None;
            }
        }

        if let Err(err) = backend.save_to_file(&Path::new(CONFIG_DIRECTORY).join(BACKEND_FILE), false) {
            error!("Failed to save backend configuration to file: {}", err);
        }

        Some(backend)
    }

    pub fn node_exists(&self, _name: &str) -> bool {
        true
    }
}

impl SaveToTomlFile for Backend {}
impl LoadFromTomlFile for Backend {}