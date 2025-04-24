use color_eyre::eyre::Result;
use tonic::Streaming;

use crate::application::network::proto::{
    common::notify,
    manage::{
        group, node,
        resource::{DelReq, SetReq},
        screen, server,
        transfer::TransferReq,
        user,
    },
};

use super::EstablishedConnection;

impl EstablishedConnection {
    pub async fn request_stop(&mut self) -> Result<()> {
        let request = self.create_request(());
        self.connection.write().await.request_stop(request).await?;
        Ok(())
    }

    pub async fn set_resource(&mut self, request: SetReq) -> Result<()> {
        let request = self.create_request(request);
        self.connection.write().await.set_resource(request).await?;
        Ok(())
    }

    pub async fn delete_resource(&mut self, request: DelReq) -> Result<()> {
        let request = self.create_request(request);
        self.connection
            .write()
            .await
            .delete_resource(request)
            .await?;
        Ok(())
    }

    pub async fn get_plugins(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .get_plugins(request)
            .await?
            .into_inner()
            .plugins)
    }

    pub async fn create_node(&mut self, node: node::Item) -> Result<()> {
        let request = self.create_request(node);
        self.connection.write().await.create_node(request).await?;
        Ok(())
    }

    pub async fn update_node(&mut self, request: node::UpdateReq) -> Result<node::Item> {
        let request = self.create_request(request);
        Ok(self
            .connection
            .write()
            .await
            .update_node(request)
            .await?
            .into_inner())
    }

    pub async fn get_node(&mut self, name: &str) -> Result<node::Item> {
        let request = self.create_request(name.to_string());
        Ok(self
            .connection
            .write()
            .await
            .get_node(request)
            .await?
            .into_inner())
    }

    pub async fn get_nodes(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .get_nodes(request)
            .await?
            .into_inner()
            .nodes)
    }

    pub async fn create_group(&mut self, group: group::Item) -> Result<()> {
        let request = self.create_request(group);
        self.connection.write().await.create_group(request).await?;
        Ok(())
    }

    pub async fn update_group(&mut self, request: group::UpdateReq) -> Result<group::Item> {
        let request = self.create_request(request);
        Ok(self
            .connection
            .write()
            .await
            .update_group(request)
            .await?
            .into_inner())
    }

    pub async fn get_group(&mut self, name: &str) -> Result<group::Item> {
        let request = self.create_request(name.to_string());
        Ok(self
            .connection
            .write()
            .await
            .get_group(request)
            .await?
            .into_inner())
    }

    pub async fn get_groups(&mut self) -> Result<Vec<String>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .get_groups(request)
            .await?
            .into_inner()
            .groups)
    }

    pub async fn get_server(&mut self, uuid: &str) -> Result<server::Detail> {
        let request = self.create_request(uuid.to_string());
        Ok(self
            .connection
            .write()
            .await
            .get_server(request)
            .await?
            .into_inner())
    }

    pub async fn get_servers(&mut self) -> Result<Vec<server::Short>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .get_servers(request)
            .await?
            .into_inner()
            .servers)
    }

    pub async fn write_to_screen(&mut self, write: screen::WriteReq) -> Result<()> {
        let request = self.create_request(write);
        self.connection
            .write()
            .await
            .write_to_screen(request)
            .await?;
        Ok(())
    }

    pub async fn subscribe_to_screen(&mut self, id: &str) -> Result<Streaming<screen::Lines>> {
        let request = self.create_request(id.to_owned());
        Ok(self
            .connection
            .write()
            .await
            .subscribe_to_screen(request)
            .await?
            .into_inner())
    }

    pub async fn get_users(&mut self) -> Result<Vec<user::Item>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .get_users(request)
            .await?
            .into_inner()
            .users)
    }

    pub async fn transfer_users(&mut self, request: TransferReq) -> Result<()> {
        let request = self.create_request(request);
        self.connection
            .write()
            .await
            .transfer_users(request)
            .await?;
        Ok(())
    }

    pub async fn get_proto_ver(&mut self) -> Result<u32> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .get_proto_ver(request)
            .await?
            .into_inner())
    }

    pub async fn get_ctrl_ver(&mut self) -> Result<String> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .get_ctrl_ver(request)
            .await?
            .into_inner())
    }

    pub async fn subscribe_to_power_events(&mut self) -> Result<Streaming<notify::PowerEvent>> {
        let request = self.create_request(());
        Ok(self
            .connection
            .write()
            .await
            .subscribe_to_power_events(request)
            .await?
            .into_inner())
    }
}
