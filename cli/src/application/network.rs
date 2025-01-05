use anyhow::Result;
use proto::{
    admin_service_client::AdminServiceClient,
    cloudlet_management::CloudletValue,
    deployment_management::DeploymentValue,
    unit_management::{SimpleUnitValue, UnitValue},
};
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
    pub client: CloudConnection,
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
        let mut client = Self::from_profile(profile);
        client.connect().await?;

        let protocol = match client.get_protocol_version().await {
            Ok(version) => version,
            Err(error) => {
                warn!("<yellow>⚠</> Failed to get protocol version: {}", error);
                0
            }
        };

        Ok(EstablishedConnection {
            client,
            outdated: protocol != VERSION.protocol,
            protocol,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.client = Some(AdminServiceClient::connect(self.address.to_string()).await?);
        Ok(())
    }

    pub async fn request_stop(&mut self) -> Result<()> {
        let request = self.create_request(());

        self.client.as_mut().unwrap().request_stop(request).await?;
        Ok(())
    }

    pub async fn get_cloudlet(&mut self, name: &str) -> Result<CloudletValue> {
        let request = self.create_request(name.to_string());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_cloudlet(request)
            .await?
            .into_inner())
    }

    pub async fn get_cloudlets(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_cloudlets(request)
            .await?
            .into_inner()
            .cloudlets)
    }

    pub async fn get_deployment(&mut self, name: &str) -> Result<DeploymentValue> {
        let request = self.create_request(name.to_string());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_deployment(request)
            .await?
            .into_inner())
    }

    pub async fn get_deployments(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_deployments(request)
            .await?
            .into_inner()
            .deployments)
    }

    pub async fn get_unit(&mut self, uuid: &str) -> Result<UnitValue> {
        let request = self.create_request(uuid.to_string());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_unit(request)
            .await?
            .into_inner())
    }

    pub async fn get_units(&mut self) -> Result<Vec<SimpleUnitValue>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_units(request)
            .await?
            .into_inner()
            .units)
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

    pub async fn get_controller_version(&mut self) -> Result<String> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_controller_version(request)
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
