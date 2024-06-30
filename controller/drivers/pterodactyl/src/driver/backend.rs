use std::path::Path;

use allocation::BAllocation;
use anyhow::Result;
use colored::Colorize;
use common::{BBody, BList, BObject};
use node::BNode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use server::{BCServer, BCServerAllocation, BServer, BServerEgg, BServerFeatureLimits, BSignal};
use user::BUser;

use crate::{
    config::{LoadFromTomlFile, SaveToTomlFile, CONFIG_DIRECTORY},
    debug, error,
    exports::node::driver::bridge::Allocation,
    node::driver::http::{send_http_request, Header, Method, Response},
    warn,
};

use super::node::server::{PanelServer, ServerName};

pub mod allocation;
mod common;
mod node;
pub mod server;
mod user;

const BACKEND_FILE: &str = "backend.toml";

/* Endpoints */
const APPLICATION_ENDPOINT: &str = "/api/application";
const CLIENT_ENDPOINT: &str = "/api/client";

#[derive(Deserialize, Serialize)]
pub struct ResolvedValues {
    pub user: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Backend {
    url: Option<String>,
    tokens: Tokens,
    user: Option<String>,
    resolved: Option<ResolvedValues>,
}

#[derive(Deserialize, Serialize)]
pub struct Tokens {
    pub application: Option<String>,
    pub client: Option<String>,
}

pub enum Endpoint {
    Application,
    Client,
}

impl ResolvedValues {
    fn new_resolved(backend: &Backend) -> Result<Self> {
        let user = backend
            .get_user_by_name(backend.user.as_ref().unwrap())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "The provided user {} does not exist in the Pterodactyl panel",
                    &backend.user.as_ref().unwrap()
                )
            })?
            .id;
        Ok(Self { user })
    }
}

impl Backend {
    fn new_empty() -> Self {
        Self {
            url: Some("".to_string()),
            tokens: Tokens {
                application: Some("".to_string()),
                client: Some("".to_string()),
            },
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
                error!(
                    "Failed to save default backend configuration to file: {}",
                    &error
                );
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
        if let Ok(token) = std::env::var("PTERODACTYL_APPLICATION_TOKEN") {
            backend.tokens.application = Some(token);
        }
        if let Ok(token) = std::env::var("PTERODACTYL_CLIENT_TOKEN") {
            backend.tokens.client = Some(token);
        }
        if let Ok(user) = std::env::var("PTERODACTYL_USER") {
            backend.user = Some(user);
        }

        let mut missing = vec![];
        if backend.url.is_none() || backend.url.as_ref().unwrap().is_empty() {
            missing.push("url");
        }
        if backend.tokens.application.is_none() || backend.tokens.application.as_ref().unwrap().is_empty() {
            missing.push("tokens.application");
        }
        if backend.tokens.client.is_none() || backend.tokens.client.as_ref().unwrap().is_empty() {
            missing.push("tokens.client");
        }
        if backend.user.is_none() || backend.user.as_ref().unwrap().is_empty() {
            missing.push("user");
        }
        if !missing.is_empty() {
            error!(
                "The following required configuration values are missing: {}",
                missing.join(", ").red()
            );
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

    pub fn start_server(&self, identifier: &str) -> bool {
        self.change_power_state(identifier, "start")
    }

    pub fn restart_server(&self, identifier: &PanelServer) -> bool {
        self.change_power_state(&identifier.identifier, "restart")
    }

    pub fn stop_server(&self, server: &PanelServer) -> bool {
        self.change_power_state(&server.identifier, "stop")
    }

    fn change_power_state(&self, identifier: &str, state: &str) -> bool {
        let state = serde_json::to_vec(&BSignal {
            signal: state.to_string(),
        }).ok();
        self.send_to_api(Method::Post, &Endpoint::Client, &format!("servers/{}/power", &identifier), 204, state.as_deref(), None)
    }

    pub fn delete_server(&self, server: u32) -> bool {
        self.delete_in_api(&Endpoint::Application, &format!("servers/{}", server))
    }

    pub fn create_server(
        &self,
        name: &ServerName,
        node: u32,
        allocation: &Allocation,
        allocations: &[BAllocation],
        egg: BServerEgg,
        features: BServerFeatureLimits,
    ) -> Option<BServer> {
        let backend_server = BCServer {
            name: name.generate(),
            node,
            user: self.resolved.as_ref().unwrap().user,
            egg: egg.id,
            docker_image: allocation.deployment.image.clone(),
            startup: egg.startup.to_owned(),
            environment: allocation
                .deployment
                .environment
                .iter()
                .map(|value| (value.key.clone(), value.value.clone()))
                .collect(),
            limits: allocation.resources.into(),
            feature_limits: features,
            allocation: BCServerAllocation::from(allocations),
            start_on_completion: true,
        };
        self.post_object_to_api::<BCServer, BServer>(
            &Endpoint::Application,
            "servers",
            &backend_server,
        )
        .map(|data| data.attributes)
    }

    pub fn get_server_by_name(&self, name: &ServerName) -> Option<BServer> {
        self.api_find_on_pages::<BServer>(Method::Get, &Endpoint::Application, "servers", |object| {
            object
                .data
                .iter()
                .find(|server| server.attributes.name == name.generate())
                .map(|server| server.attributes.clone())
        })
    }

    pub fn get_free_allocations(
        &self,
        used_allocations: &[BAllocation],
        node_id: u32,
        amount: u32,
    ) -> Vec<BAllocation> {
        let mut allocations = Vec::with_capacity(amount as usize);
        self.for_each_on_pages::<BAllocation>(
            Method::Get,
            &Endpoint::Application,
            format!("nodes/{}/allocations", &node_id).as_str(),
            |response| {
                for allocation in &response.data {
                    if allocation.attributes.assigned
                        || used_allocations.iter().any(|used| {
                            used.ip == allocation.attributes.ip
                                && used.port == allocation.attributes.port
                        })
                    {
                        continue;
                    }
                    allocations.push(allocation.attributes.clone());
                    if allocations.len() >= amount as usize {
                        return true;
                    }
                }
                false
            },
        );
        allocations
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<BUser> {
        self.api_find_on_pages::<BUser>(Method::Get, &Endpoint::Application, "users", |object| {
            object
                .data
                .iter()
                .find(|node| node.attributes.username == username)
                .map(|node| node.attributes.clone())
        })
    }

    pub fn get_node_by_name(&self, name: &str) -> Option<BNode> {
        self.api_find_on_pages::<BNode>(Method::Get, &Endpoint::Application, "nodes", |object| {
            object
                .data
                .iter()
                .find(|node| node.attributes.name == name)
                .map(|node| node.attributes.clone())
        })
    }

    fn api_find_on_pages<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &Endpoint,
        target: &str,
        mut callback: impl FnMut(&BBody<Vec<BObject<T>>>) -> Option<T>,
    ) -> Option<T> {
        let mut value = None;
        self.for_each_on_pages(method, endpoint, target, |response| {
            if let Some(data) = callback(response) {
                value = Some(data);
                return true;
            }
            false
        });
        value
    }

    fn for_each_on_pages<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &Endpoint,
        target: &str,
        mut callback: impl FnMut(&BBody<Vec<BObject<T>>>) -> bool,
    ) {
        let mut page = 1;
        loop {
            if let Some(response) = self.api_get_list::<T>(method, endpoint, target, Some(page)) {
                if callback(&response) {
                    return;
                }
                if response.meta.pagination.total_pages <= page {
                    break;
                }
                page += 1;
            }
        }
    }

    fn api_get_list<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &Endpoint,
        target: &str,
        page: Option<u32>,
    ) -> Option<BList<T>> {
        self.send_to_api_parse(method, endpoint, target, 200, None, page)
    }

    fn delete_in_api(&self, endpoint: &Endpoint, target: &str) -> bool {
        self.send_to_api(Method::Delete, endpoint, target, 204, None, None)
    }

    fn post_object_to_api<T: Serialize, K: DeserializeOwned>(
        &self,
        endpoint: &Endpoint,
        target: &str,
        object: &T,
    ) -> Option<BObject<K>> {
        let body = serde_json::to_vec(object).ok();
        self.send_to_api_parse(Method::Post, endpoint, target, 201, body.as_deref(), None)
    }

    fn send_to_api(
        &self,
        method: Method,
        endpoint: &Endpoint,
        target: &str,
        expected_code: u32,
        body: Option<&[u8]>,
        page: Option<u32>,
    ) -> bool {
        let mut url = format!("{}{}/{}", &self.url.as_ref().unwrap(), match endpoint {
            Endpoint::Application => APPLICATION_ENDPOINT,
            Endpoint::Client => CLIENT_ENDPOINT,
        }, target);
        if let Some(page) = page {
            url = format!("{}?page={}", &url, &page);
        }
        debug!(
            "Sending request to the pterodactyl panel: {:?} {}",
            method, &url
        );
        let response = send_http_request(
            method,
            &url,
            &[
                Header {
                    key: "Authorization".to_string(),
                    value: format!("Bearer {}", match endpoint {
                        Endpoint::Application => self.tokens.application.as_ref().unwrap(),
                        Endpoint::Client => self.tokens.client.as_ref().unwrap(),
                    }),
                },
                Header {
                    key: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
            ],
            body,
        );
        if Self::check_response(&url, body, response, expected_code).is_some() {
            return true;
        }
        false
    }

    fn send_to_api_parse<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &Endpoint,
        target: &str,
        expected_code: u32,
        body: Option<&[u8]>,
        page: Option<u32>,
    ) -> Option<T> {
        let mut url = format!("{}{}/{}", &self.url.as_ref().unwrap(), match endpoint {
            Endpoint::Application => APPLICATION_ENDPOINT,
            Endpoint::Client => CLIENT_ENDPOINT,
        }, target);
        if let Some(page) = page {
            url = format!("{}?page={}", &url, &page);
        }
        debug!(
            "Sending request to the pterodactyl panel: {:?} {}",
            method, &url
        );
        let response = send_http_request(
            method,
            &url,
            &[
                Header {
                    key: "Authorization".to_string(),
                    value: format!("Bearer {}", match endpoint {
                        Endpoint::Application => self.tokens.application.as_ref().unwrap(),
                        Endpoint::Client => self.tokens.client.as_ref().unwrap(),
                    }),
                },
                Header {
                    key: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                },
            ],
            body,
        );
        if let Some(response) = Self::handle_response::<T>(&url, body, response, expected_code) {
            return Some(response);
        }
        None
    }

    fn check_response(
        url: &str,
        body: Option<&[u8]>,
        response: Option<Response>,
        expected_code: u32,
    ) -> Option<Response> {
        response.as_ref()?;
        let response = response.unwrap();
        if response.status_code != expected_code {
            error!(
                    "An unexpected error occurred while sending a request to the Pterodactyl panel at {}: Received {} status code {} - {}",
                    url,
                    response.status_code,
                    response.reason_phrase,
                    String::from_utf8_lossy(&response.bytes)
                );
            if let Some(body) = body {
                debug!("Sended body: {}", String::from_utf8_lossy(body));
            }
            debug!(
                "Response body: {}",
                String::from_utf8_lossy(&response.bytes)
            );
            return None;
        }
        Some(response)
    }

    fn handle_response<T: DeserializeOwned>(
        url: &str,
        body: Option<&[u8]>,
        response: Option<Response>,
        expected_code: u32,
    ) -> Option<T> {
        let response = Self::check_response(url, body, response, expected_code)?;
        let response = serde_json::from_slice::<T>(&response.bytes);
        if let Err(error) = response {
            error!(
                "Failed to parse response from the Pterodactyl panel at URL {}: {}",
                url, &error
            );
            return None;
        }
        Some(response.unwrap())
    }
}

impl SaveToTomlFile for Backend {}
impl LoadFromTomlFile for Backend {}