use anyhow::Result;
use log::error;
use std::{net::SocketAddr, process::exit};

use proto::server_service_client::ServerServiceClient;
use tonic::{transport::Channel, Request};

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("server");
}

pub struct CloudConnection {
    /* Data */
    address: SocketAddr,
    token: String,

    /* TLS */
    tls_config: Option<String>,

    /* Client */
    client: Option<ServerServiceClient<Channel>>,
}

impl CloudConnection {
    pub fn from_env() -> Self {
        let address;
        let token;
        let tls_config;

        if let Ok(value) = std::env::var("CONTROLLER_ADDRESS") {
            if let Ok(value) = value.parse::<SocketAddr>() {
                address = Some(value);
            } else {
                error!("Failed to parse CONTROLLER_ADDRESS environment variable");
                exit(1);
            }
        } else {
            error!("Missing CONTROLLER_ADDRESS environment variable. Please set it to the address of the controller");
            exit(1);
        }

        if let Ok(value) = std::env::var("SERVER_TOKEN") {
            token = Some(value);
        } else {
            error!("Missing SERVER_TOKEN environment variable. Please set it to the token of this server");
            exit(1);
        }

        if let Ok(value) = std::env::var("CONTROLLER_TLS_CONFIG") {
            tls_config = Some(value);
        } else {
            tls_config = None;
        }

        Self::new(address.unwrap(), token.unwrap(), tls_config)
    }

    pub fn new(address: SocketAddr, token: String, tls_config: Option<String>) -> Self {
        Self {
            address,
            token,
            tls_config,
            client: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.client = Some(
            ServerServiceClient::connect(format!(
                "{}://{}",
                if self.tls_config.is_some() {
                    "https"
                } else {
                    "http"
                },
                self.address
            ))
            .await?,
        );
        Ok(())
    }

    pub fn create_request<T>(&self, data: T) -> Request<T> {
        let mut request = Request::new(data);

        // Add the token to the metadata
        request
            .metadata_mut()
            .insert("authorization", self.token.parse().unwrap());

        request
    }

    pub async fn beat_heart(&mut self) -> Result<()> {
        let request = self.create_request(());

        self.client.as_mut().unwrap().beat_heart(request).await?;
        Ok(())
    }
}
