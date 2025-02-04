use anyhow::Result;
use beat::BeatTask;
use common::error::CloudError;
use health::{RequestStopTask, SetRunningTask};
use ready::SetReadyTask;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::{
    application::TaskSender,
    task::{BoxedTask, Task},
    VERSION,
};

use super::proto::client::{
    self,
    channel::Msg,
    client_service_server::ClientService,
    transfer::{TransferReq, TransferRes},
    user::{ConnectedReq, DisconnectedReq},
};

mod beat;
mod group;
mod health;
mod ready;
mod reset;
mod server;
mod transfer;
mod user;

pub struct ClientServiceImpl(pub TaskSender);

#[async_trait]
impl ClientService for ClientServiceImpl {
    type SubscribeToTransfersStream = ReceiverStream<Result<TransferRes, Status>>;
    type SubscribeToChannelStream = ReceiverStream<Result<Msg, Status>>;

    // Heartbeat
    async fn beat(&self, mut request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            self.execute_task::<(), _, _>(&mut request, |_, server| Box::new(BeatTask { server }))
                .await?,
        ))
    }

    // Ready state
    async fn set_ready(&self, mut request: Request<bool>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            self.execute_task::<(), _, _>(&mut request, |request, server| {
                Box::new(SetReadyTask {
                    server,
                    ready: *request.get_ref(),
                })
            })
            .await?,
        ))
    }

    // Health
    async fn set_running(&self, mut request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            self.execute_task::<(), _, _>(&mut request, |_, server| {
                Box::new(SetRunningTask { server })
            })
            .await?,
        ))
    }
    async fn request_stop(&self, mut request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            self.execute_task::<(), _, _>(&mut request, |_, server| {
                Box::new(RequestStopTask { server })
            })
            .await?,
        ))
    }

    // User
    async fn user_connected(
        &self,
        _request: Request<ConnectedReq>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }
    async fn user_disconnected(
        &self,
        _request: Request<DisconnectedReq>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    // Transfer
    async fn transfer_users(
        &self,
        _request: Request<TransferReq>,
    ) -> Result<Response<u32>, Status> {
        todo!()
    }
    async fn subscribe_to_transfers(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::SubscribeToTransfersStream>, Status> {
        todo!()
    }

    // Channel
    async fn publish_message(&self, _request: Request<Msg>) -> Result<Response<u32>, Status> {
        todo!()
    }
    async fn subscribe_to_channel(
        &self,
        _request: Request<String>,
    ) -> Result<Response<Self::SubscribeToChannelStream>, Status> {
        todo!()
    }

    // Server
    async fn get_servers(
        &self,
        _request: Request<()>,
    ) -> Result<Response<client::server::List>, Status> {
        todo!()
    }

    // Group
    async fn get_groups(
        &self,
        _request: Request<()>,
    ) -> Result<Response<client::group::List>, Status> {
        todo!()
    }

    // Housekeeping
    async fn reset(&self, _request: Request<()>) -> Result<Response<()>, Status> {
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

impl ClientServiceImpl {
    async fn execute_task<O: Send + 'static, I, F>(
        &self,
        request: &mut Request<I>,
        task: F,
    ) -> Result<O, Status>
    where
        F: FnOnce(&mut Request<I>, Uuid) -> BoxedTask,
    {
        let server = *match request.extensions().get::<Uuid>() {
            Some(server) => server,
            None => return Err(Status::permission_denied("Not linked to server")),
        };
        match Task::create::<O>(&self.0, task(request, server)).await {
            Ok(value) => Ok(value),
            Err(error) => {
                CloudError::print_fancy(&error, false);
                Err(Status::internal(error.to_string()))
            }
        }
    }
}
