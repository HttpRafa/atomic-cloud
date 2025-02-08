use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use beat::BeatTask;
use health::{RequestStopTask, SetRunningTask};
use ready::SetReadyTask;
use server::GetServersTask;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request, Response, Status};
use transfer::TransferUsersTask;
use user::{UserConnectedTask, UserDisconnectedTask};
use uuid::Uuid;

use crate::{
    application::{
        auth::AuthType, server::NameAndUuid, user::transfer::TransferTarget, Shared, TaskSender,
    },
    task::Task,
    VERSION,
};

use super::proto::client::{
    self,
    channel::Msg,
    client_service_server::ClientService,
    transfer::{target::Type, TransferReq, TransferRes},
    user::{ConnectedReq, DisconnectedReq},
};

mod beat;
mod group;
mod health;
mod ready;
mod server;
mod transfer;
mod user;

pub type TransferMsg = TransferRes;
pub type ChannelMsg = Msg;

pub struct ClientServiceImpl(pub TaskSender, pub Arc<Shared>);

#[async_trait]
impl ClientService for ClientServiceImpl {
    type SubscribeToTransfersStream = ReceiverStream<Result<TransferRes, Status>>;
    type SubscribeToChannelStream = ReceiverStream<Result<Msg, Status>>;

    // Heartbeat
    async fn beat(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::Server, &self.0, request, |_, auth| {
                Ok(Box::new(BeatTask(auth)))
            })
            .await?,
        ))
    }

    // Ready state
    async fn set_ready(&self, request: Request<bool>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::Server, &self.0, request, |request, auth| {
                Ok(Box::new(SetReadyTask(
                    auth,
                    *request.get_ref(),
                )))
            })
            .await?,
        ))
    }

    // Health
    async fn set_running(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::Server, &self.0, request, |_, auth| {
                Ok(Box::new(SetRunningTask(auth)))
            })
            .await?,
        ))
    }
    async fn request_stop(&self, request: Request<()>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::Server, &self.0, request, |_, auth| {
                Ok(Box::new(RequestStopTask(auth)))
            })
            .await?,
        ))
    }

    // User
    async fn user_connected(&self, request: Request<ConnectedReq>) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::Server, &self.0, request, |request, auth| {
                let request = request.into_inner();

                let name = request.name;
                let uuid = match Uuid::from_str(&request.id) {
                    Ok(uuid) => uuid,
                    Err(_) => return Err(Status::invalid_argument("Invalid UUID")),
                };

                Ok(Box::new(UserConnectedTask(auth, NameAndUuid::new(name, uuid))))
            })
            .await?,
        ))
    }
    async fn user_disconnected(
        &self,
        request: Request<DisconnectedReq>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(
            Task::execute::<(), _, _>(AuthType::Server, &self.0, request, |request, auth| {
                let request = request.into_inner();

                let uuid = match Uuid::from_str(&request.id) {
                    Ok(uuid) => uuid,
                    Err(_) => return Err(Status::invalid_argument("Invalid UUID")),
                };

                Ok(Box::new(UserDisconnectedTask(auth, uuid)))
            })
            .await?,
        ))
    }

    // Transfer
    async fn transfer_users(&self, request: Request<TransferReq>) -> Result<Response<u32>, Status> {
        Ok(Response::new(
            Task::execute::<u32, _, _>(AuthType::Server, &self.0, request, |request, auth| {
                let request = request.into_inner();

                let target = match request.target {
                    Some(target) => match Type::try_from(target.r#type) {
                        Ok(r#type) => match (target.target, r#type) {
                            (Some(target), Type::Group) => TransferTarget::Group(target),
                            (Some(target), Type::Server) => {
                                TransferTarget::Server(match Uuid::from_str(&target) {
                                    Ok(uuid) => uuid,
                                    Err(_) => return Err(Status::invalid_argument("Invalid UUID")),
                                })
                            }
                            (None, Type::Fallback) => TransferTarget::Fallback,
                            _ => {
                                return Err(Status::invalid_argument(
                                    "Invalid target type combination",
                                ))
                            }
                        },
                        Err(_) => return Err(Status::invalid_argument("Invalid target type")),
                    },
                    None => return Err(Status::invalid_argument("Missing target")),
                };
                let uuids = request
                    .ids
                    .into_iter()
                    .map(|id| match Uuid::from_str(&id) {
                        Ok(uuid) => Ok(uuid),
                        Err(_) => Err(Status::invalid_argument("Invalid UUID")),
                    })
                    .collect::<Result<Vec<Uuid>, _>>()?;

                Ok(Box::new(TransferUsersTask(
                    auth,
                    uuids,
                    target,
                )))
            })
            .await?,
        ))
    }
    async fn subscribe_to_transfers(
        &self,
        request: Request<()>,
    ) -> Result<Response<Self::SubscribeToTransfersStream>, Status> {
        let auth = Task::get_auth(AuthType::Server, &request)?;
        let server = auth
            .get_server()
            .expect("Should be ok. Because type is checked in get_auth");

        Ok(Response::new(
            self.1.subscribers.subscribe_transfer(*server.uuid()).await,
        ))
    }

    // Channel
    async fn publish_message(&self, request: Request<Msg>) -> Result<Response<u32>, Status> {
        Ok(Response::new(
            self.1
                .subscribers
                .publish_channel(request.into_inner())
                .await,
        ))
    }
    async fn subscribe_to_channel(
        &self,
        request: Request<String>,
    ) -> Result<Response<Self::SubscribeToChannelStream>, Status> {
        Ok(Response::new(
            self.1
                .subscribers
                .subscribe_channel(request.into_inner())
                .await,
        ))
    }

    // Server
    async fn get_servers(
        &self,
        request: Request<()>,
    ) -> Result<Response<client::server::List>, Status> {
        Ok(Response::new(
            Task::execute::<client::server::List, _, _>(
                AuthType::Server,
                &self.0,
                request,
                |_, _| Ok(Box::new(GetServersTask())),
            )
            .await?,
        ))
    }

    // Group
    async fn get_groups(
        &self,
        _request: Request<()>,
    ) -> Result<Response<client::group::List>, Status> {
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
