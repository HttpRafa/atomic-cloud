use std::path::Path;

use colored::Colorize;
use node::BNodes;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    config::{LoadFromTomlFile, SaveToTomlFile, CONFIG_DIRECTORY}, debug, error, node::driver::{http::{send_http_request, Header, Method, Response}, log::{request_user_input, Question}}, warn
};

mod node;

const BACKEND_FILE: &str = "backend.toml";

const APPLICATION_ENDPOINT: &str = "/api/application";

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
            backend.url.as_ref()?;
        }

        if backend.token.is_none() {
            backend.token = request_user_input(Question::Password, "What is the token of the pelican panel?", &[]);
            backend.token.as_ref()?;
        }

        if let Err(err) = backend.save_to_file(&Path::new(CONFIG_DIRECTORY).join(BACKEND_FILE), false) {
            error!("Failed to save backend configuration to file: {}", err);
        }

        Some(backend)
    }

    pub fn node_exists(&self, name: &str) -> bool {
        if let Some(response) = self.from_api::<BNodes>(Method::Get, APPLICATION_ENDPOINT, "nodes") {
            return response.data.iter().any(|node| node.attributes.name == name);
        }
        false
    }

    fn from_api<T: DeserializeOwned>(&self, method: Method, endpoint: &str, target: &str) -> Option<T> {
        let url = format!("{}{}/{}", self.url.as_ref().unwrap(), endpoint, target);
        debug!("Sending request to the pelican panel: {:?} {}", method, &url);
        let response = send_http_request(method, &url, &[Header {
            key: "Authorization".to_string(),
            value: format!("Bearer {}", self.token.as_ref().unwrap()),
        }]);
        if let Some(response) = Self::handle_response::<T>(response, 200) {
            return Some(response);
        }
        None
    }

    fn handle_response<T: DeserializeOwned>(response: Option<Response>, expected_code: u32) -> Option<T> {
        if response.is_none() {
            return None;
        }
        let response = response.unwrap();
        if response.status_code != expected_code {
            error!("Received {} status code {} from the pelican panel: {}", "unexpected".red(), &response.status_code, &response.reason_phrase);
            debug!("Response body: {}", String::from_utf8_lossy(&response.bytes));
            return None;
        }
        let response = serde_json::from_slice::<T>(&response.bytes);
        if let Err(error) = response {
            error!("{} to parse response from the pelican panel: {}", "Failed".red(), &error);
            return None;
        }
        Some(response.unwrap())
    }
}

impl SaveToTomlFile for Backend {}
impl LoadFromTomlFile for Backend {}