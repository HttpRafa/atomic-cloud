use std::path::Path;

use allocation::BAllocation;
use anyhow::Result;
use colored::Colorize;
use common::{BBody, BList, BObject};
use node::BNode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use user::BUser;

use crate::{
    config::{LoadFromTomlFile, SaveToTomlFile, CONFIG_DIRECTORY}, debug, error, node::driver::http::{send_http_request, Header, Method, Response}, warn
};

mod common;
mod node;
mod user;
mod allocation;

const BACKEND_FILE: &str = "backend.toml";

/* Endpoints */
const APPLICATION_ENDPOINT: &str = "/api/application";

#[derive(Deserialize, Serialize)]
pub struct ResolvedValues {
    pub user: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Backend {
    url: Option<String>,
    token: Option<String>,
    user: Option<String>,
    resolved: Option<ResolvedValues>,
}

impl ResolvedValues {
    fn new_resolved(backend: &Backend) -> Result<Self> {
        let user = backend.get_user_by_name(&backend.user.as_ref().unwrap()).ok_or_else(|| anyhow::anyhow!("The provided user {} does not exist in the Pterodactyl panel", &backend.user.as_ref().unwrap()))?.id;
        Ok(Self {
            user,
        })
    }
}

impl Backend {
    fn new_empty() -> Self {
        Self {
            url: Some("".to_string()),
            token: Some("".to_string()),
            user: Some("".to_string()),
            resolved: None,
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
            let backend = Self::new_empty();     
            if let Err(error) = backend.save_to_file(&path, false) {
                error!("Failed to save default backend configuration to file: {}", &error);
            }
            backend
        }
    }

    fn new_filled() -> Result<Self> {
        let mut backend = Self::load_or_empty();

        // Check config values are overridden by environment variables
        if let Ok(url) = std::env::var("PTERODACTYL_URL") {
            backend.url = Some(url);
        }
        if let Ok(token) = std::env::var("PTERODACTYL_TOKEN") {
            backend.token = Some(token);
        }
        if let Ok(user) = std::env::var("PTERODACTYL_USER") {
            backend.user = Some(user);
        }

        let mut missing = vec![];
        if backend.url.is_none() || backend.url.as_ref().unwrap().is_empty() {
            missing.push("url");
        }
        if backend.token.is_none() || backend.token.as_ref().unwrap().is_empty() {
            missing.push("token");
        }
        if backend.user.is_none() || backend.user.as_ref().unwrap().is_empty() {
            missing.push("user");
        }
        if !missing.is_empty() {
            error!("The following required configuration values are missing: {}", missing.join(", ").red());
            return Err(anyhow::anyhow!("Missing required configuration values"));
        }

        Ok(backend)
    }

    pub fn new_filled_and_resolved() -> Result<Self> {
        let mut backend = Self::new_filled()?;
        match ResolvedValues::new_resolved(&backend) {
            Ok(resolved) => backend.resolved = Some(resolved),
            Err(error) => return Err(error),
        }
        Ok(backend)
    }

    pub fn get_free_allocations(&self, node_id: u32, amount: u32) -> Vec<BAllocation> {
        let mut allocations = Vec::with_capacity(amount as usize);
        self.api_find_on_pages::<BAllocation>(Method::Get, APPLICATION_ENDPOINT, format!("nodes/{}/allocations", &node_id).as_str(), |object| {
            for allocation in &object.data {
                if allocation.attributes.assigned {
                    continue;
                }
                allocations.push(allocation.attributes.clone());
                if allocations.len() >= amount as usize {
                    return None;
                }
            }
            None
        });
        allocations
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<BUser> {
        self.api_find_on_pages::<BUser>(Method::Get, APPLICATION_ENDPOINT, "users", |object| 
            object.data.iter().find(|node| node.attributes.username == username).map(|node| node.attributes.clone()))
    }

    pub fn get_node_by_name(&self, name: &str) -> Option<BNode> {
        self.api_find_on_pages::<BNode>(Method::Get, APPLICATION_ENDPOINT, "nodes", |object| 
            object.data.iter().find(|node| node.attributes.name == name).map(|node| node.attributes.clone()))
    }

    fn api_find_on_pages<T: DeserializeOwned>(&self, method: Method, endpoint: &str, target: &str, mut callback: impl FnMut(&BBody<Vec<BObject<T>>>) -> Option<T>) -> Option<T> {
        let mut page = 1;
        loop {
            if let Some(response) = self.api_get_list::<T>(method, endpoint, target, Some(page)) {
                if let Some(data) = callback(&response) {
                    return Some(data);
                }
                if response.meta.pagination.total_pages <= page {
                    break;
                }
                page += 1;
            }
        }
        None
    }

    fn api_get_list<T: DeserializeOwned>(&self, method: Method, endpoint: &str, target: &str, page: Option<u32>) -> Option<BList<T>> {
        self.api_get::<Vec<BObject<T>>>(method, endpoint, target, page)
    }

    fn api_get<T: DeserializeOwned>(&self, method: Method, endpoint: &str, target: &str, page: Option<u32>) -> Option<BBody<T>> {
        let mut url = format!("{}{}/{}", &self.url.as_ref().unwrap(), endpoint, target);
        if let Some(page) = page {
            url = format!("{}?page={}", &url, &page);
        }
        debug!("Sending request to the pterodactyl panel: {:?} {}", method, &url);
        let response = send_http_request(method, &url, &[Header {
            key: "Authorization".to_string(),
            value: format!("Bearer {}", &self.token.as_ref().unwrap()),
        }]);
        if let Some(response) = Self::handle_response::<BBody<T>>(response, 200) {
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