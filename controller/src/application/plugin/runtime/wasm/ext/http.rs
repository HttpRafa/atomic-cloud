use anyhow::{anyhow, Result};
use simplelog::warn;
use tokio::task::spawn_blocking;

use crate::application::plugin::runtime::wasm::{
    config::Permissions,
    generated::plugin::system::{
        self,
        http::{Header, Method, Response},
    },
    PluginState,
};

impl system::http::Host for PluginState {
    // TODO: Rewrite this function to use the reqwest crate instead of minreq
    async fn send_http_request(
        &mut self,
        method: Method,
        url: String,
        headers: Vec<Header>,
        body: Option<Vec<u8>>,
    ) -> Result<Option<Response>> {
        // Check if the plugin has permissions
        if !self.permissions.contains(Permissions::ALLOW_HTTP) {
            return Err(anyhow!(
                "Plugin tried to send a http request without the required permissions"
            ));
        }

        let name = self.name.clone();
        Ok(spawn_blocking(move || {
            let mut request = match method {
                Method::Get => minreq::get(url),
                Method::Patch => minreq::patch(url),
                Method::Post => minreq::post(url),
                Method::Put => minreq::put(url),
                Method::Delete => minreq::delete(url),
            };
            if let Some(body) = body {
                request = request.with_body(body);
            }
            for header in headers {
                request = request.with_header(&header.key, &header.value);
            }
            let response = match request.send() {
                Ok(response) => response,
                Err(error) => {
                    warn!("Failed to send HTTP request for plugin {}: {}", name, error);
                    return None;
                }
            };
            Some(Response {
                #[allow(clippy::cast_sign_loss)]
                status_code: response.status_code as u32,
                reason_phrase: response.reason_phrase.clone(),
                headers: response
                    .headers
                    .iter()
                    .map(|header| Header {
                        key: header.0.clone(),
                        value: header.1.clone(),
                    })
                    .collect(),
                bytes: response.into_bytes(),
            })
        })
        .await
        .ok()
        .flatten())
    }
}
