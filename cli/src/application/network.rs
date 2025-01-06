use std::fmt::Display;

use anyhow::Result;
use proto::{
    admin_service_client::AdminServiceClient,
    cloudlet_management::CloudletValue,
    deployment_management::DeploymentValue,
    resource_management::{
        DeleteResourceRequest, ResourceCategory, ResourceStatus, SetResourceStatusRequest,
    },
    unit_management::{unit_spec::Retention, SimpleUnitValue, UnitValue},
    user_management::{transfer_target_value::TargetType, TransferUserRequest, UserValue},
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
    pub incompatible: bool,
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
            incompatible: protocol != VERSION.protocol,
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

    pub async fn set_resource_status(&mut self, request: SetResourceStatusRequest) -> Result<()> {
        let request = self.create_request(request);
        self.client
            .as_mut()
            .unwrap()
            .set_resource_status(request)
            .await?;
        Ok(())
    }

    pub async fn delete_resource(&mut self, request: DeleteResourceRequest) -> Result<()> {
        let request = self.create_request(request);
        self.client
            .as_mut()
            .unwrap()
            .delete_resource(request)
            .await?;
        Ok(())
    }

    pub async fn get_drivers(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_drivers(request)
            .await?
            .into_inner()
            .drivers)
    }

    pub async fn create_cloudlet(&mut self, cloudlet: CloudletValue) -> Result<()> {
        let request = self.create_request(cloudlet);
        self.client
            .as_mut()
            .unwrap()
            .create_cloudlet(request)
            .await?;
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

    pub async fn create_deployment(&mut self, deployment: DeploymentValue) -> Result<()> {
        let request = self.create_request(deployment);
        self.client
            .as_mut()
            .unwrap()
            .create_deployment(request)
            .await?;
        Ok(())
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

    pub async fn get_users(&mut self) -> Result<Vec<UserValue>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_users(request)
            .await?
            .into_inner()
            .users)
    }

    pub async fn transfer_user(
        &mut self,
        transfer_user_request: TransferUserRequest,
    ) -> Result<()> {
        let request = self.create_request(transfer_user_request);
        self.client.as_mut().unwrap().transfer_user(request).await?;
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

impl Display for Retention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Retention::Temporary => write!(f, "Temporary"),
            Retention::Permanent => write!(f, "Permanent"),
        }
    }
}

impl Display for UserValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.uuid)
    }
}

impl Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::Unit => write!(f, "Unit"),
            TargetType::Deployment => write!(f, "Deployment"),
        }
    }
}

impl Display for ResourceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceCategory::Cloudlet => write!(f, "Cloudlet"),
            ResourceCategory::Deployment => write!(f, "Deployment"),
            ResourceCategory::Unit => write!(f, "Unit"),
        }
    }
}

impl Display for ResourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceStatus::Active => write!(f, "Active"),
            ResourceStatus::Inactive => write!(f, "Inactive"),
        }
    }
}
