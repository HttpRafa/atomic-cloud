use anyhow::Result;
use common::error::CloudError;
use power::RequestStopTask;
use tonic::{async_trait, Request, Response, Status};

use crate::{
    application::{auth::AdminUser, TaskSender},
    task::{BoxedTask, Task},
    VERSION,
};

use super::proto::manage::{
    self,
    manage_service_server::ManageService,
    resource::{DelReq, SetReq},
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

pub struct ManageServiceImpl(pub TaskSender);

#[async_trait]
impl ManageService for ManageServiceImpl {
    // Power
    async fn request_stop(&self, mut request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            self.execute_task::<(), _, _>(&mut request, |_, _| Box::new(RequestStopTask()))
                .await?,
        ))
    }

    // Resource
    async fn set_resource(&self, _request: Request<SetReq>) -> Result<Response<()>, Status> {
        todo!()
    }
    async fn delete_resource(&self, _request: Request<DelReq>) -> Result<Response<()>, Status> {
        todo!()
    }

    // Plugin
    async fn get_plugins(
        &self,
        _request: Request<()>,
    ) -> Result<Response<manage::plugin::List>, Status> {
        todo!()
    }

    // Node
    async fn create_node(
        &self,
        _request: Request<manage::node::Item>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }
    async fn get_node(
        &self,
        _request: Request<String>,
    ) -> Result<Response<manage::node::Item>, Status> {
        todo!()
    }
    async fn get_nodes(
        &self,
        _request: Request<()>,
    ) -> Result<Response<manage::node::List>, Status> {
        todo!()
    }

    // Group
    async fn create_group(
        &self,
        _request: Request<manage::group::Item>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }
    async fn get_group(
        &self,
        _request: Request<String>,
    ) -> Result<Response<manage::group::Item>, Status> {
        todo!()
    }
    async fn get_groups(
        &self,
        _request: Request<()>,
    ) -> Result<Response<manage::group::List>, Status> {
        todo!()
    }

    // Server
    async fn get_server(
        &self,
        _request: Request<String>,
    ) -> Result<Response<manage::server::Detail>, Status> {
        todo!()
    }
    async fn get_servers(
        &self,
        _request: Request<()>,
    ) -> Result<Response<manage::server::List>, Status> {
        todo!()
    }

    // User
    async fn get_users(
        &self,
        _request: Request<()>,
    ) -> Result<Response<manage::user::List>, Status> {
        todo!()
    }

    // Transfer
    async fn transfer_users(
        &self,
        _request: Request<TransferReq>,
    ) -> Result<Response<u32>, Status> {
        todo!()
    }

    // Version info
    async fn get_proto_ver(&self, _request: Request<()>) -> Result<Response<u32>, Status> {
        Ok(Response::new(VERSION.protocol))
    }
    async fn get_ctrl_ver(&self, _request: Request<()>) -> Result<Response<String>, Status> {
        Ok(Response::new(format!("{}", VERSION)))
    }
}

impl ManageServiceImpl {
    async fn execute_task<O: Send + 'static, I, F>(
        &self,
        request: &mut Request<I>,
        task: F,
    ) -> Result<O, Status>
    where
        F: FnOnce(&mut Request<I>, AdminUser) -> BoxedTask,
    {
        let server = match request.extensions().get::<AdminUser>() {
            Some(server) => server,
            None => return Err(Status::permission_denied("Not linked to user")),
        }
        .clone();
        match Task::create::<O>(&self.0, task(request, server)).await {
            Ok(value) => Ok(value),
            Err(error) => {
                CloudError::print_fancy(&error, false);
                Err(Status::internal(error.to_string()))
            }
        }
    }
}
