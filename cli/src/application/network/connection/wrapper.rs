use color_eyre::eyre::Result;
use tonic::Streaming;

use crate::application::network::proto::manage::{
    group, node,
    resource::{DelReq, SetReq},
    screen, server,
    transfer::TransferReq,
    user,
};

use super::EstablishedConnection;

impl EstablishedConnection {
    pub async fn request_stop(&mut self) -> Result<()> {
        let request = self.create_request(());
        self.connection.request_stop(request).await?;
        Ok(())
    }

    pub async fn set_resource(&mut self, request: SetReq) -> Result<()> {
        let request = self.create_request(request);
        self.connection.set_resource(request).await?;
        Ok(())
    }

    pub async fn delete_resource(&mut self, request: DelReq) -> Result<()> {
        let request = self.create_request(request);
        self.connection.delete_resource(request).await?;
        Ok(())
    }

    pub async fn get_plugins(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .get_plugins(request)
            .await?
            .into_inner()
            .plugins)
    }

    pub async fn create_node(&mut self, node: node::Item) -> Result<()> {
        let request = self.create_request(node);
        self.connection.create_node(request).await?;
        Ok(())
    }

    pub async fn get_node(&mut self, name: &str) -> Result<node::Item> {
        let request = self.create_request(name.to_string());
        Ok(self.connection.get_node(request).await?.into_inner())
    }

    pub async fn get_nodes(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self.connection.get_nodes(request).await?.into_inner().nodes)
    }

    pub async fn create_group(&mut self, group: group::Item) -> Result<()> {
        let request = self.create_request(group);
        self.connection.create_group(request).await?;
        Ok(())
    }

    pub async fn get_group(&mut self, name: &str) -> Result<group::Item> {
        let request = self.create_request(name.to_string());
        Ok(self.connection.get_group(request).await?.into_inner())
    }

    pub async fn get_groups(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .get_groups(request)
            .await?
            .into_inner()
            .groups)
    }

    pub async fn get_server(&mut self, uuid: &str) -> Result<server::Detail> {
        let request = self.create_request(uuid.to_string());
        Ok(self.connection.get_server(request).await?.into_inner())
    }

    pub async fn get_servers(&mut self) -> Result<Vec<server::Short>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .get_servers(request)
            .await?
            .into_inner()
            .servers)
    }

    pub async fn write_to_screen(&mut self, write: screen::WriteReq) -> Result<()> {
        let request = self.create_request(write);
        self.connection.write_to_screen(request).await?;
        Ok(())
    }

    pub async fn subscribe_to_screen(&mut self, id: &str) -> Result<Streaming<screen::Lines>> {
        let request = self.create_request(id.to_owned());
        Ok(self
            .connection
            .subscribe_to_screen(request)
            .await?
            .into_inner())
    }

    pub async fn get_users(&mut self) -> Result<Vec<user::Item>> {
        let request = self.create_request(());
        Ok(self.connection.get_users(request).await?.into_inner().users)
    }

    pub async fn transfer_users(&mut self, request: TransferReq) -> Result<()> {
        let request = self.create_request(request);
        self.connection.transfer_users(request).await?;
        Ok(())
    }

    pub async fn get_proto_ver(&mut self) -> Result<u32> {
        let request = self.create_request(());
        Ok(self.connection.get_proto_ver(request).await?.into_inner())
    }

    pub async fn get_ctrl_ver(&mut self) -> Result<String> {
        let request = self.create_request(());
        Ok(self.connection.get_ctrl_ver(request).await?.into_inner())
    }
}
