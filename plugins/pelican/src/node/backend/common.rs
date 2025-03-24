use data::{BBody, BList, BObject};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    debug, error,
    generated::plugin::system::http::{send_http_request, Header, Method, Response},
};

use super::{Backend, Endpoint};

pub mod data;

/* Endpoints */
const APPLICATION_ENDPOINT: &str = "api/application";
const CLIENT_ENDPOINT: &str = "api/client";

impl Backend {
    pub fn api_find_on_pages<T: DeserializeOwned>(
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

    pub fn for_each_on_pages<T: DeserializeOwned>(
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
                if response.meta.is_none() || response.meta.unwrap().pagination.total_pages <= page
                {
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

    pub fn delete_in_api(&self, endpoint: &Endpoint, target: &str) -> bool {
        self.send_to_api(Method::Delete, endpoint, target, 204, None, None)
    }

    pub fn get_object_from_api<T: Serialize, K: DeserializeOwned>(
        &self,
        endpoint: &Endpoint,
        target: &str,
        object: &T,
    ) -> Option<BObject<K>> {
        let body = serde_json::to_vec(object).ok();
        self.send_to_api_parse(Method::Get, endpoint, target, 200, body.as_deref(), None)
    }

    pub fn post_object_to_api<T: Serialize, K: DeserializeOwned>(
        &self,
        endpoint: &Endpoint,
        target: &str,
        object: &T,
    ) -> Option<BObject<K>> {
        let body = serde_json::to_vec(object).ok();
        self.send_to_api_parse(Method::Post, endpoint, target, 201, body.as_deref(), None)
    }

    pub fn send_to_api(
        &self,
        method: Method,
        endpoint: &Endpoint,
        target: &str,
        expected_code: u32,
        body: Option<&[u8]>,
        page: Option<u32>,
    ) -> bool {
        let mut url = format!(
            "{}{}/{}",
            self.url,
            match endpoint {
                Endpoint::Application => APPLICATION_ENDPOINT,
                Endpoint::Client => CLIENT_ENDPOINT,
            },
            target
        );
        if let Some(page) = page {
            url = format!("{}?page={}", &url, &page);
        }
        debug!(
            "Sending request to the pelican panel: {:?} {}",
            method, &url
        );
        let response = send_http_request(
            method,
            &url,
            &[
                Header {
                    key: "Authorization".to_string(),
                    value: format!(
                        "Bearer {}",
                        match endpoint {
                            Endpoint::Application => &self.token,
                            Endpoint::Client => &self.user_token,
                        }
                    ),
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
        let mut url = format!(
            "{}{}/{}",
            self.url,
            match endpoint {
                Endpoint::Application => APPLICATION_ENDPOINT,
                Endpoint::Client => CLIENT_ENDPOINT,
            },
            target
        );
        if let Some(page) = page {
            url = format!("{}?page={}", &url, &page);
        }
        debug!(
            "Sending request to the pelican panel: {:?} {}",
            method, &url
        );
        let response = send_http_request(
            method,
            &url,
            &[
                Header {
                    key: "Authorization".to_string(),
                    value: format!(
                        "Bearer {}",
                        match endpoint {
                            Endpoint::Application => &self.token,
                            Endpoint::Client => &self.user_token,
                        }
                    ),
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
                    "An unexpected error occurred while sending a request to the Pelican panel at {}: Received {} status code {} - {}",
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
                "Failed to parse response from the Pelican panel at URL {}: {}",
                url, &error
            );
            return None;
        }
        Some(response.unwrap())
    }
}
