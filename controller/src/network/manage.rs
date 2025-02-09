use std::sync::Arc;

use anyhow::Result;
use group::{CreateGroupTask, GetGroupTask, GetGroupsTask};
use node::{CreateNodeTask, GetNodeTask, GetNodesTask};
use plugin::GetPluginsTask;
use power::RequestStopTask;
use resource::{DeleteResourceTask, SetResourceTask};
use server::{GetServerTask, GetServersTask};
use tonic::{async_trait, Request, Response, Status};
use transfer::TransferUsersTask;
use user::GetUsersTask;

use crate::{
    application::{auth::AuthType, Shared, TaskSender},
    task::Task,
    VERSION,
};

use super::proto::manage::{
    self,
    manage_service_server::ManageService,
    resource::{Category, DelReq, SetReq},
    transfer::TransferReq,
};

mod group;
mod node;
mod plugin;
mod power;
mod resource;
mod server;
mod transfer;
mod user;

pub struct ManageServiceImpl(pub TaskSender, pub Arc<Shared>);

#[async_trait]
impl ManageService for ManageServiceImpl {
    // Power
    async fn request_stop(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(RequestStopTask()))
            })
            .await?,
        ))
    }

    // Resource
    async fn set_resource(&self, request: Request<SetReq>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |request, _| {
                let request = request.into_inner();

                let category = match Category::try_from(request.category) {
                    Ok(category) => category,
                    Err(_) => {
                        return Err(Status::invalid_argument("Invalid category provided"));
                    }
                };

                Ok(Box::new(SetResourceTask(
                    category,
                    request.id,
                    request.active,
                )))
            })
            .await?,
        ))
    }
    async fn delete_resource(&self, request: Request<DelReq>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |request, _| {
                let request = request.into_inner();

                let category = match Category::try_from(request.category) {
                    Ok(category) => category,
                    Err(_) => {
                        return Err(Status::invalid_argument("Invalid category provided"));
                    }
                };

                Ok(Box::new(DeleteResourceTask(category, request.id)))
            })
            .await?,
        ))
    }

    // Plugin
    async fn get_plugins(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::plugin::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::plugin::List, _, _>(
                AuthType::User,
                &self.0,
                request,
                |_, _| Ok(Box::new(GetPluginsTask())),
            )
            .await?,
        ))
    }

    // Node
    async fn create_node(
        &self,
        request: Request<manage::node::Item>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(CreateNodeTask()))
            })
            .await?,
        ))
    }
    async fn get_node(
        &self,
        request: Request<String>,
    ) -> Result<Response<manage::node::Item>, Status> {
        Ok(Response::new(
            Task::execute::<manage::node::Item, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetNodeTask()))
            })
            .await?,
        ))
    }
    async fn get_nodes(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::node::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::node::List, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetNodesTask()))
            })
            .await?,
        ))
    }

    // Group
    async fn create_group(
        &self,
        request: Request<manage::group::Item>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(CreateGroupTask()))
            })
            .await?,
        ))
    }
    async fn get_group(
        &self,
        request: Request<String>,
    ) -> Result<Response<manage::group::Item>, Status> {
        Ok(Response::new(
            Task::execute::<manage::group::Item, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetGroupTask()))
            })
            .await?,
        ))
    }
    async fn get_groups(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::group::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::group::List, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetGroupsTask()))
            })
            .await?,
        ))
    }

    // Server
    async fn get_server(
        &self,
        request: Request<String>,
    ) -> Result<Response<manage::server::Detail>, Status> {
        Ok(Response::new(
            Task::execute::<manage::server::Detail, _, _>(
                AuthType::User,
                &self.0,
                request,
                |_, _| Ok(Box::new(GetServerTask())),
            )
            .await?,
        ))
    }
    async fn get_servers(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::server::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::server::List, _, _>(
                AuthType::User,
                &self.0,
                request,
                |_, _| Ok(Box::new(GetServersTask())),
            )
            .await?,
        ))
    }

    // User
    async fn get_users(
        &self,
        request: Request<()>,
    ) -> Result<Response<manage::user::List>, Status> {
        Ok(Response::new(
            Task::execute::<manage::user::List, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(GetUsersTask()))
            })
            .await?,
        ))
    }

    // Transfer
    async fn transfer_users(&self, request: Request<TransferReq>) -> Result<Response<u32>, Status> {
        Ok(Response::new(
            Task::execute::<u32, _, _>(AuthType::User, &self.0, request, |_, _| {
                Ok(Box::new(TransferUsersTask()))
            })
            .await?,
        ))
    }

    // Version info
    async fn get_proto_ver(&self, _request: Request<()>) -> Result<Response<u32>, Status> {
        Ok(Response::new(VERSION.protocol))
    }
    async fn get_ctrl_ver(&self, _request: Request<()>) -> Result<Response<String>, Status> {
        Ok(Response::new(format!("{}", VERSION)))
    }
}
