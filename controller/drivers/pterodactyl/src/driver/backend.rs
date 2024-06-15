use std::path::Path;

use colored::Colorize;
use node::BNodes;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    config::{LoadFromTomlFile, SaveToTomlFile, CONFIG_DIRECTORY}, debug, error, node::driver::http::{send_http_request, Header, Method, Response}, warn
};

mod node;

const BACKEND_FILE: &str = "backend.toml";

/* Endpoints */
const APPLICATION_ENDPOINT: &str = "/api/application";

#[derive(Deserialize, Serialize, Default)]
pub struct Backend {
    url: Option<String>,
    token: Option<String>,
}

impl Backend {
    fn load_or_empty() -> Self {
        let path = Path::new(CONFIG_DIRECTORY).join(BACKEND_FILE);
        if path.exists() {
            Self::load_from_file(&path).unwrap_or_else(|err| {
                warn!("Failed to read backend configuration from file: {}", err);
                Self::default()
            })
        } else {
            Self::default()
        }
    }

    pub fn new_filled() -> Option<Self> {
        let mut backend = Self::load_or_empty();
        let mut save = false;

        if backend.url.is_none() {
            backend.url = Some("http://panel.gameserver.example.com".to_string());
            save = true;
        }

        if backend.token.is_none() {
            backend.token = Some("yourToken".to_string());
            save = true;
        }

        if save {
            if let Err(error) = backend.save_to_file(&Path::new(CONFIG_DIRECTORY).join(BACKEND_FILE), false) {
                error!("Failed to save backend configuration to file: {}", &error);
            }
        }

        // Check config values are overridden by environment variables
        if let Ok(url) = std::env::var("PTERODACTYL_URL") {
            backend.url = Some(url);
        }
        if let Ok(token) = std::env::var("PTERODACTYL_TOKEN") {
            backend.token = Some(token);
        }

        Some(backend)
    }

    pub fn node_exists(&self, name: &str) -> bool {
        if let Some(response) = self.pull_api::<BNodes>(Method::Get, APPLICATION_ENDPOINT, "nodes") {
            return response.data.iter().any(|node| node.attributes.name == name);
        }
        false
    }

    fn pull_api<T: DeserializeOwned>(&self, method: Method, endpoint: &str, target: &str) -> Option<T> {
        let url = format!("{}{}/{}", self.url.as_ref().unwrap(), endpoint, target);
        debug!("Sending request to the pterodactyl panel: {:?} {}", method, &url);
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
        response.as_ref()?;
        let response = response.unwrap();
        if response.status_code != expected_code {
            error!("Received {} status code {} from the pterodactyl panel: {}", "unexpected".red(), &response.status_code, &response.reason_phrase);
            debug!("Response body: {}", String::from_utf8_lossy(&response.bytes));
            return None;
        }
        let response = serde_json::from_slice::<T>(&response.bytes);
        if let Err(error) = response {
            error!("{} to parse response from the pterodactyl panel: {}", "Failed".red(), &error);
            return None;
        }
        Some(response.unwrap())
    }
}

impl SaveToTomlFile for Backend {}
impl LoadFromTomlFile for Backend {}