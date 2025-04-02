use std::{process::exit, sync::Arc};

use anyhow::Result;
use proto::manage::{
    client_service_client::ClientServiceClient,
    transfer::TransferRes,
    user::{ConnectedReq, DisconnectedReq},
};
use simplelog::error;
use tokio::sync::Mutex;
use url::Url;

use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig},
    Request, Response, Status, Streaming,
};

pub mod proto {
    pub mod manage {
        #![allow(clippy::all, clippy::pedantic)]
        use tonic::include_proto;

        include_proto!("client");
    }

    pub mod common {
        #![allow(clippy::all, clippy::pedantic)]
        use tonic::include_proto;

        include_proto!("common");
    }
}

pub type CloudConnectionHandle = Arc<CloudConnection>;

pub struct CloudConnection {
    /* Data */
    address: Url,
    token: String,

    /* TLS */
    cert: Option<String>,

    /* Client */
    client: Mutex<Option<ClientServiceClient<Channel>>>,
}

impl CloudConnection {
    pub fn from_env() -> Arc<Self> {
        let address;
        let token;
        let cert;

        if let Ok(value) = std::env::var("CONTROLLER_ADDRESS") {
            if let Ok(value) = Url::parse(&value) {
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
            error!(
                "Missing SERVER_TOKEN environment variable. Please set it to the token of this server"
            );
            exit(1);
        }

        if let Ok(value) = std::env::var("CONTROLLER_CERTIFICATE") {
            cert = Some(value);
        } else {
            cert = None;
        }

        Arc::new(Self::new(address.unwrap(), token.unwrap(), cert))
    }

    pub fn new(address: Url, token: String, cert: Option<String>) -> Self {
        Self {
            address,
            token,
            cert,
            client: Mutex::new(None),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        let mut tls = ClientTlsConfig::new().with_enabled_roots();
        if let Some(cert) = &self.cert {
            tls = tls.ca_certificate(Certificate::from_pem(cert));
        }
        let channel = Channel::from_shared(self.address.to_string())?
            .tls_config(tls)?
            .connect()
            .await?;
        *self.client.lock().await = Some(ClientServiceClient::new(channel));
        Ok(())
    }

    pub async fn beat(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .expect("No connection created")
            .beat(request)
            .await?;
        Ok(())
    }

    pub async fn set_ready(&self, ready: bool) -> Result<()> {
        let request = self.create_request(ready);

        self.client
            .lock()
            .await
            .as_mut()
            .expect("No connection created")
            .set_ready(request)
            .await?;
        Ok(())
    }

    pub async fn set_running(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .expect("No connection created")
            .set_running(request)
            .await?;
        Ok(())
    }

    pub async fn request_stop(&self) -> Result<()> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .expect("No connection created")
            .request_stop(request)
            .await?;
        Ok(())
    }

    pub async fn user_connected(&self, name: String, id: String) -> Result<()> {
        let request = self.create_request(ConnectedReq { name, id });

        self.client
            .lock()
            .await
            .as_mut()
            .expect("No connection created")
            .user_connected(request)
            .await?;
        Ok(())
    }

    pub async fn user_disconnected(&self, id: String) -> Result<()> {
        let request = self.create_request(DisconnectedReq { id });

        self.client
            .lock()
            .await
            .as_mut()
            .expect("No connection created")
            .user_disconnected(request)
            .await?;
        Ok(())
    }

    pub async fn subscribe_to_transfers(&self) -> Result<Response<Streaming<TransferRes>>, Status> {
        let request = self.create_request(());

        self.client
            .lock()
            .await
            .as_mut()
            .expect("No connection created")
            .subscribe_to_transfers(request)
            .await
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
