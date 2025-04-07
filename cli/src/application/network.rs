use std::fmt::Display;

use anyhow::Result;
use proto::manage::{
    cloudGroup,
    manage_service_client::ManageServiceClient,
    node,
    resource::{self, DelReq, SetReq},
    screen,
    server::{self, DiskRetention},
    transfer::{self, target, TransferReq},
    user,
};
use simplelog::warn;
use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig},
    Request, Streaming,
};
use url::Url;

use crate::VERSION;

use super::profile::Profile;

pub mod proto {
    pub mod manage {
        #![allow(clippy::all, clippy::pedantic)]
        use tonic::include_proto;

        include_proto!("manage");
    }

    pub mod common {
        #![allow(clippy::all, clippy::pedantic)]
        use tonic::include_proto;

        include_proto!("common");
    }
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
    cert: Option<String>,

    /* Client */
    client: Option<ManageServiceClient<Channel>>,
}

impl CloudConnection {
    pub fn from_profile(profile: &Profile) -> Self {
        Self::new(
            profile.url.clone(),
            profile.authorization.clone(),
            profile.cert.clone(),
        )
    }

    pub fn new(address: Url, token: String, cert: Option<String>) -> Self {
        Self {
            address,
            token,
            cert,
            client: None,
        }
    }

    pub async fn establish_connection(profile: &Profile) -> Result<EstablishedConnection> {
        let mut client = Self::from_profile(profile);
        client.connect().await?;

        let protocol = match client.get_proto_ver().await {
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
        let mut tls = ClientTlsConfig::new().with_enabled_roots();
        if let Some(cert) = &self.cert {
            tls = tls.ca_certificate(Certificate::from_pem(cert));
        }
        let channel = Channel::from_shared(self.address.to_string())?
            .tls_config(tls)?
            .connect()
            .await?;
        self.client = Some(ManageServiceClient::new(channel));
        Ok(())
    }

    pub async fn request_stop(&mut self) -> Result<()> {
        let request = self.create_request(());
        self.client.as_mut().unwrap().request_stop(request).await?;
        Ok(())
    }

    pub async fn set_resource(&mut self, request: SetReq) -> Result<()> {
        let request = self.create_request(request);
        self.client.as_mut().unwrap().set_resource(request).await?;
        Ok(())
    }

    pub async fn delete_resource(&mut self, request: DelReq) -> Result<()> {
        let request = self.create_request(request);
        self.client
            .as_mut()
            .unwrap()
            .delete_resource(request)
            .await?;
        Ok(())
    }

    pub async fn get_plugins(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_plugins(request)
            .await?
            .into_inner()
            .plugins)
    }

    pub async fn create_node(&mut self, node: node::Item) -> Result<()> {
        let request = self.create_request(node);
        self.client.as_mut().unwrap().create_node(request).await?;
        Ok(())
    }

    pub async fn get_node(&mut self, name: &str) -> Result<node::Item> {
        let request = self.create_request(name.to_string());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_node(request)
            .await?
            .into_inner())
    }

    pub async fn get_nodes(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_nodes(request)
            .await?
            .into_inner()
            .nodes)
    }

    pub async fn create_group(&mut self, cloudGroup: cloudGroup::Item) -> Result<()> {
        let request = self.create_request(cloudGroup);
        self.client.as_mut().unwrap().create_group(request).await?;
        Ok(())
    }

    pub async fn get_group(&mut self, name: &str) -> Result<cloudGroup::Item> {
        let request = self.create_request(name.to_string());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_group(request)
            .await?
            .into_inner())
    }

    pub async fn get_groups(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_groups(request)
            .await?
            .into_inner()
            .groups)
    }

    pub async fn get_server(&mut self, uuid: &str) -> Result<server::Detail> {
        let request = self.create_request(uuid.to_string());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_server(request)
            .await?
            .into_inner())
    }

    pub async fn get_servers(&mut self) -> Result<Vec<server::Short>> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_servers(request)
            .await?
            .into_inner()
            .servers)
    }

    pub async fn write_to_screen(&mut self, write: screen::WriteReq) -> Result<()> {
        let request = self.create_request(write);
        self.client
            .as_mut()
            .unwrap()
            .write_to_screen(request)
            .await?;
        Ok(())
    }

    pub async fn subscribe_to_screen(&mut self, id: &str) -> Result<Streaming<screen::Lines>> {
        let request = self.create_request(id.to_owned());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .subscribe_to_screen(request)
            .await?
            .into_inner())
    }

    pub async fn get_users(&mut self) -> Result<Vec<user::Item>> {
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

    pub async fn transfer_users(&mut self, request: TransferReq) -> Result<()> {
        let request = self.create_request(request);
        self.client
            .as_mut()
            .unwrap()
            .transfer_users(request)
            .await?;
        Ok(())
    }

    pub async fn get_proto_ver(&mut self) -> Result<u32> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_proto_ver(request)
            .await?
            .into_inner())
    }

    pub async fn get_ctrl_ver(&mut self) -> Result<String> {
        let request = self.create_request(());
        Ok(self
            .client
            .as_mut()
            .unwrap()
            .get_ctrl_ver(request)
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

impl Display for DiskRetention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskRetention::Temporary => write!(f, "Temporary"),
            DiskRetention::Permanent => write!(f, "Permanent"),
        }
    }
}

impl Display for user::Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

impl Display for server::Short {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

impl Display for target::Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            target::Type::Server => write!(f, "Server"),
            target::Type::Group => write!(f, "Group"),
            target::Type::Fallback => write!(f, "Fallback"),
        }
    }
}

impl Display for transfer::Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match target::Type::try_from(self.r#type)
            .expect("There is something wrong with the target type")
        {
            target::Type::Server => write!(
                f,
                "Server ({})",
                self.target.as_ref().unwrap_or(&String::from("None"))
            ),
            target::Type::Group => write!(
                f,
                "Group ({})",
                self.target.as_ref().unwrap_or(&String::from("None"))
            ),
            target::Type::Fallback => write!(f, "Fallback"),
        }
    }
}

impl Display for resource::Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            resource::Category::Node => write!(f, "Node"),
            resource::Category::Group => write!(f, "Group"),
            resource::Category::Server => write!(f, "Server"),
        }
    }
}
