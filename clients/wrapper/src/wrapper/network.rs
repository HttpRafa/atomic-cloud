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
    /* Address */
    address: SocketAddr,

    /* Client */
    client: Option<ServerServiceClient<Channel>>,
}

impl CloudConnection {
    pub fn from_env() -> Self {
        if let Ok(address) = std::env::var("CLOUD_ADDRESS") {
            if let Ok(address) = address.parse() {
                return Self::new(address);
            } else {
                error!("Failed to parse CLOUD_ADDRESS environment variable");
                exit(1);
            }
        } else {
            error!("Missing CLOUD_ADDRESS environment variable. Please set it to the address of the controller");
            exit(1);
        }
    }

    pub fn new(address: SocketAddr) -> Self {
        Self {
            address,
            client: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.client = Some(ServerServiceClient::connect(format!("http://{}", self.address)).await?);
        Ok(())
    }

    pub async fn beat_heart(&mut self) -> Result<()> {
        self.client
            .as_mut()
            .unwrap()
            .beat_heart(Request::new(()))
            .await?;
        Ok(())
    }
}
