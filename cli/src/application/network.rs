use anyhow::Result;
use proto::admin_service_client::AdminServiceClient;
use simplelog::warn;
use tonic::{transport::Channel, Request};
use url::Url;

use crate::VERSION;

use super::profile::Profile;

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("admin");
}

pub struct EstablishedConnection {
    pub connection: CloudConnection,
    pub outdated: bool,
    pub protocol: u32,
}

pub struct CloudConnection {
    /* Data */
    address: Url,
    token: String,

    /* TLS */
    //tls_config: Option<String>,

    /* Client */
    client: Option<AdminServiceClient<Channel>>,
}

impl CloudConnection {
    pub fn from_profile(profile: &Profile) -> Self {
        Self::new(profile.url.clone(), profile.authorization.clone())
    }

    pub fn new(address: Url, token: String) -> Self {
        Self {
            address,
            token,
            client: None,
        }
    }

    pub async fn establish_connection(profile: &Profile) -> Result<EstablishedConnection> {
        let mut connection = Self::from_profile(profile);
        connection.connect().await?;

        let protocol = match connection.get_protocol_version().await {
            Ok(version) => version,
            Err(error) => {
                warn!("<yellow>⚠</> Failed to get protocol version: {}", error);
                0
            }
        };

        Ok(EstablishedConnection {
            connection,
            outdated: protocol != VERSION.protocol,
            protocol,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.client = Some(AdminServiceClient::connect(self.address.to_string()).await?);
        Ok(())
    }

    pub async fn get_protocol_version(&mut self) -> Result<u32> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_protocol_version(request)
            .await?
            .into_inner())
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
