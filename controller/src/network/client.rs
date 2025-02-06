use std::str::FromStr;

use anyhow::Result;
use beat::BeatTask;
use health::{RequestStopTask, SetRunningTask};
use ready::SetReadyTask;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request, Response, Status};
use transfer::TransferUsersTask;
use user::{UserConnectedTask, UserDisconnectedTask};
use uuid::Uuid;

use crate::{application::TaskSender, task::Task, VERSION};

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
    async fn beat(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), Uuid, _, _>(&self.0, request, |_, server| {
                Ok(Box::new(BeatTask { server }))
            })
            .await?,
        ))
    }

    // Ready state
    async fn set_ready(&self, request: Request<bool>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), Uuid, _, _>(&self.0, request, |request, server| {
                Ok(Box::new(SetReadyTask {
                    server,
                    ready: *request.get_ref(),
                }))
            })
            .await?,
        ))
    }

    // Health
    async fn set_running(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), Uuid, _, _>(&self.0, request, |_, server| {
                Ok(Box::new(SetRunningTask { server }))
            })
            .await?,
        ))
    }
    async fn request_stop(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), Uuid, _, _>(&self.0, request, |_, server| {
                Ok(Box::new(RequestStopTask { server }))
            })
            .await?,
        ))
    }

    // User
    async fn user_connected(&self, request: Request<ConnectedReq>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), Uuid, _, _>(&self.0, request, |request, server| {
                let request = request.into_inner();

                let name = request.name;
                let uuid = match Uuid::from_str(&request.id) {
                    Ok(uuid) => uuid,
                    Err(_) => return Err(Status::invalid_argument("Invalid UUID")),
                };

                Ok(Box::new(UserConnectedTask { server, name, uuid }))
            })
            .await?,
        ))
    }
    async fn user_disconnected(
        &self,
        request: Request<DisconnectedReq>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(Task::execute::<(), Uuid, _, _>(
            &self.0,
            request,
            |request, server| {
                let request = request.into_inner();

                let uuid = match Uuid::from_str(&request.id) {
                    Ok(uuid) => uuid,
                    Err(_) => return Err(Status::invalid_argument("Invalid UUID")),
                };

                Ok(Box::new(UserDisconnectedTask { server, uuid }))
            },
        ).await?))
    }

    // Transfer
    async fn transfer_users(&self, request: Request<TransferReq>) -> Result<Response<u32>, Status> {
        Ok(Response::new(Task::execute::<u32, Uuid, _, _>(
            &self.0,
            request,
            |_, server| Ok(Box::new(TransferUsersTask { server })),
        ).await?))
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
