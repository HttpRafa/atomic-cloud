use simplelog::warn;

use super::{
    generated::cloudlet::driver::{
        self,
        http::{Header, Method, Response},
    },
    WasmDriverState,
};

impl driver::http::Host for WasmDriverState {
    fn send_http_request(
        &mut self,
        method: Method,
        url: String,
        headers: Vec<Header>,
        body: Option<Vec<u8>>,
    ) -> Option<Response> {
        let driver = self.handle.upgrade().unwrap();
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
                warn!(
                    "<red>Failed</> to send HTTP request for driver <blue>{}</>: <red>{}</>",
                    &driver.name, error
                );
                return None;
            }
        };
        Some(Response {
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
    }
}
