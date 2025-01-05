use std::{process::exit, sync::Arc};

use anyhow::Result;
use simplelog::error;
use tokio::sync::Mutex;
use url::Url;

use proto::{
    transfer_management::ResolvedTransferResponse,
    unit_service_client::UnitServiceClient,
    user_management::{UserConnectedRequest, UserDisconnectedRequest},
};
use tonic::{transport::Channel, Request, Response, Status, Streaming};

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("unit");
}

pub type CloudConnectionHandle = Arc<CloudConnection>;

pub struct CloudConnection {
    /* Data */
    address: Url,
    token: String,

    /* TLS */
    //tls_config: Option<String>,

    /* Client */
    client: Mutex<Option<UnitServiceClient<Channel>>>,
}

impl CloudConnection {
    pub fn from_env() -> Arc<Self> {
        let address;
        let token;
        let tls_config;

        if let Ok(value) = std::env::var("CONTROLLER_ADDRESS") {
            if let Ok(value) = Url::parse(&value) {
                address = Some(value);
            } else {
                error!("<red>Failed</> to parse CONTROLLER_ADDRESS environment variable");
                exit(1);
            }
        } else {
            error!("<red>Missing</> CONTROLLER_ADDRESS environment variable. Please set it to the address of the controller");
            exit(1);
        }

        if let Ok(value) = std::env::var("UNIT_TOKEN") {
            token = Some(value);
        } else {
            error!(
                "<red>Missing</> UNIT_TOKEN environment variable. Please set it to the token of this unit"
            );
            exit(1);
        }

        if let Ok(value) = std::env::var("CONTROLLER_TLS_CONFIG") {
            tls_config = Some(value);
        } else {
            tls_config = None;
        }

        Arc::new(Self::new(address.unwrap(), token.unwrap(), tls_config))
    }

    pub fn new(address: Url, token: String, _tls_config: Option<String>) -> Self {
        Self {
            address,
            token,
            //tls_config,
            client: Mutex::new(None),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        *self.client.lock().await =
            Some(UnitServiceClient::connect(self.address.to_string()).await?);
        Ok(())
    }

    pub async fn beat_heart(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .beat_heart(request)
            .await?;
        Ok(())
    }

    pub async fn mark_ready(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .mark_ready(request)
            .await?;
        Ok(())
    }

    pub async fn mark_not_ready(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .mark_not_ready(request)
            .await?;
        Ok(())
    }

    pub async fn mark_running(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .mark_running(request)
            .await?;
        Ok(())
    }

    pub async fn request_stop(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .request_stop(request)
            .await?;
        Ok(())
    }

    pub async fn user_connected(&self, name: String, uuid: String) -> Result<()> {
        let request = self.create_request(UserConnectedRequest { name, uuid });

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .user_connected(request)
            .await?;
        Ok(())
    }

    pub async fn user_disconnected(&self, uuid: String) -> Result<()> {
        let request = self.create_request(UserDisconnectedRequest { uuid });

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .user_disconnected(request)
            .await?;
        Ok(())
    }

    pub async fn subscribe_to_transfers(
        &self,
    ) -> Result<Response<Streaming<ResolvedTransferResponse>>, Status> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .subscribe_to_transfers(request)
            .await
    }

    pub async fn send_reset(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .unwrap()
            .reset(request)
            .await?;
        Ok(())
    }

    fn create_request<T>(&self, data: T) -> Request<T> {
        let mut request = Request::new(data);

        // Add the token to the metadata
        request
            .metadata_mut()
            .insert("authorization", self.token.parse().unwrap());

        request
    }
}
