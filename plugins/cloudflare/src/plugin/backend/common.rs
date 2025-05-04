use data::BObject;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    debug, error,
    generated::plugin::system::http::{send_http_request, Header, Method, Response},
};

use super::Backend;

pub mod data;
pub mod error;

pub const CLOUDFLARE_API_URL: &str = "https://api.cloudflare.com/client/v4";

impl Backend {
    pub fn post_object_to_api<T: Serialize, K: DeserializeOwned>(
        &self,
        target: &str,
        object: &T,
    ) -> Option<BObject<K>> {
        let body = serde_json::to_vec(object).ok();
        self.send_to_api_parse(Method::Post, target, 200, body.as_deref(), None)
    }

    fn send_to_api_parse<T: DeserializeOwned>(
        &self,
        method: Method,
        target: &str,
        expected_code: u32,
        body: Option<&[u8]>,
        page: Option<u32>,
    ) -> Option<T> {
        let mut url = format!("{CLOUDFLARE_API_URL}/{target}");
        if let Some(page) = page {
            url = format!("{}?page={}", &url, &page);
        }
        debug!(
            "Sending request to the cloudflare api: {:?} {}",
            method, &url
        );
        let response = send_http_request(
            method,
            &url,
            &[
                Header {
                    key: "Authorization".to_string(),
                    value: format!("Bearer {}", self.token),
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
                    "An unexpected error occurred while sending a request to the cloudflare api at {}: Received {} status code {} - {}",
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
        let body = response.bytes;
        let response = serde_json::from_slice::<T>(&body);
        if let Err(error) = response {
            error!(
                "Failed to parse response from the cloudflare api at URL {}: {}",
                url, &error
            );
            debug!("Response body: {}", String::from_utf8_lossy(&body));
            return None;
        }
        Some(response.unwrap())
    }
}
