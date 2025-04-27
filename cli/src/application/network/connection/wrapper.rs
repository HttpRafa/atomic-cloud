use std::borrow::Cow;

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

use super::{
    task::{spawn, EmptyTask, NetworkTask},
    EstablishedConnection,
};

impl EstablishedConnection {
    pub fn request_stop(&self) -> EmptyTask {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            connection.write().await.request_stop(request).await?;
            Ok(())
        })
    }

    pub fn set_resource(&self, request: SetReq) -> EmptyTask {
        let connection = self.connection.clone();
        let request = self.create_request(request);

        spawn(async move {
            connection.write().await.set_resource(request).await?;
            Ok(())
        })
    }

    pub fn delete_resource(&self, request: DelReq) -> EmptyTask {
        let connection = self.connection.clone();
        let request = self.create_request(request);

        spawn(async move {
            connection.write().await.delete_resource(request).await?;
            Ok(())
        })
    }

    pub fn get_plugins(&self) -> NetworkTask<Result<Vec<String>>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_plugins(request)
                .await?
                .into_inner()
                .plugins)
        })
    }

    pub fn create_node(&self, node: node::Detail) -> EmptyTask {
        let connection = self.connection.clone();
        let request = self.create_request(node);

        spawn(async move {
            connection.write().await.create_node(request).await?;
            Ok(())
        })
    }

    pub fn update_node(&self, request: node::UpdateReq) -> NetworkTask<Result<node::Detail>> {
        let connection = self.connection.clone();
        let request = self.create_request(request);

        spawn(async move {
            Ok(connection
                .write()
                .await
                .update_node(request)
                .await?
                .into_inner())
        })
    }

    pub fn get_node<'a, T>(&self, name: T) -> NetworkTask<Result<node::Detail>>
    where
        T: Into<Cow<'a, str>>,
    {
        let connection = self.connection.clone();
        let request = self.create_request(name.into().into_owned());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_node(request)
                .await?
                .into_inner())
        })
    }

    pub fn get_nodes(&self) -> NetworkTask<Result<Vec<node::Short>>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_nodes(request)
                .await?
                .into_inner()
                .nodes)
        })
    }

    pub fn create_group(&self, group: group::Detail) -> EmptyTask {
        let connection = self.connection.clone();
        let request = self.create_request(group);

        spawn(async move {
            connection.write().await.create_group(request).await?;
            Ok(())
        })
    }

    pub fn update_group(&self, request: group::UpdateReq) -> NetworkTask<Result<group::Detail>> {
        let connection = self.connection.clone();
        let request = self.create_request(request);

        spawn(async move {
            Ok(connection
                .write()
                .await
                .update_group(request)
                .await?
                .into_inner())
        })
    }

    pub fn get_group<'a, T>(&self, name: T) -> NetworkTask<Result<group::Detail>>
    where
        T: Into<Cow<'a, str>>,
    {
        let connection = self.connection.clone();
        let request = self.create_request(name.into().into_owned());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_group(request)
                .await?
                .into_inner())
        })
    }

    pub fn get_groups(&self) -> NetworkTask<Result<Vec<group::Short>>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_groups(request)
                .await?
                .into_inner()
                .groups)
        })
    }

    pub fn get_server<'a, T>(&self, uuid: T) -> NetworkTask<Result<server::Detail>>
    where
        T: Into<Cow<'a, str>>,
    {
        let connection = self.connection.clone();
        let request = self.create_request(uuid.into().into_owned());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_server(request)
                .await?
                .into_inner())
        })
    }

    pub fn get_servers(&self) -> NetworkTask<Result<Vec<server::Short>>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_servers(request)
                .await?
                .into_inner()
                .servers)
        })
    }

    pub fn write_to_screen(&self, write: screen::WriteReq) -> EmptyTask {
        let connection = self.connection.clone();
        let request = self.create_request(write);

        spawn(async move {
            connection.write().await.write_to_screen(request).await?;
            Ok(())
        })
    }

    pub fn subscribe_to_screen<'a, T>(&self, id: T) -> NetworkTask<Result<Streaming<screen::Lines>>>
    where
        T: Into<Cow<'a, str>>,
    {
        let connection = self.connection.clone();
        let request = self.create_request(id.into().into_owned());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .subscribe_to_screen(request)
                .await?
                .into_inner())
        })
    }

    pub fn get_users(&self) -> NetworkTask<Result<Vec<user::Item>>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_users(request)
                .await?
                .into_inner()
                .users)
        })
    }

    pub fn transfer_users(&self, request: TransferReq) -> EmptyTask {
        let connection = self.connection.clone();
        let request = self.create_request(request);

        spawn(async move {
            connection.write().await.transfer_users(request).await?;
            Ok(())
        })
    }

    pub fn get_proto_ver(&self) -> NetworkTask<Result<u32>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_proto_ver(request)
                .await?
                .into_inner())
        })
    }

    pub fn get_ctrl_ver(&self) -> NetworkTask<Result<String>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .get_ctrl_ver(request)
                .await?
                .into_inner())
        })
    }

    pub fn subscribe_to_power_events(&self) -> NetworkTask<Result<Streaming<notify::PowerEvent>>> {
        let connection = self.connection.clone();
        let request = self.create_request(());

        spawn(async move {
            Ok(connection
                .write()
                .await
                .subscribe_to_power_events(request)
                .await?
                .into_inner())
        })
    }
}
