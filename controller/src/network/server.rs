use std::{pin::Pin, str::FromStr};

use proto::{server_service_server::ServerService, ChannelMessage, TransferTarget, User};
use tokio::sync::mpsc::channel;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::controller::{auth::AuthServerHandle, ControllerHandle};

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("server");
}

pub struct ServerServiceImpl {
    pub controller: ControllerHandle,
}

#[async_trait]
impl ServerService for ServerServiceImpl {
    async fn beat_heart(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            self.controller.get_servers().handle_heart_beat(&server);
            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn mark_ready(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            self.controller.get_servers().mark_ready(&server);
            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn mark_not_ready(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            self.controller.get_servers().mark_not_ready(&server);
            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn mark_running(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            self.controller.get_servers().mark_running(&server);
            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn request_stop(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            self.controller.get_servers().checked_stop_server(&server);
            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn user_connected(&self, request: Request<User>) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            let user = request.into_inner();
            self.controller.get_users().handle_user_connected(
                server,
                user.name,
                match Uuid::from_str(&user.uuid) {
                    Ok(uuid) => uuid,
                    Err(error) => {
                        return Err(Status::invalid_argument(format!(
                            "Failed to parse UUID: {}",
                            error
                        )))
                    }
                },
            );
            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn user_disconnected(&self, request: Request<User>) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            let user = request.into_inner();
            self.controller.get_users().handle_user_disconnected(
                server,
                match Uuid::from_str(&user.uuid) {
                    Ok(uuid) => uuid,
                    Err(error) => {
                        return Err(Status::invalid_argument(format!(
                            "Failed to parse UUID: {}",
                            error
                        )))
                    }
                },
            );
            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn transfer_all_users(
        &self,
        request: Request<TransferTarget>,
    ) -> Result<Response<u32>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            Ok(Response::new(
                self.controller.get_servers().transfer_all_users(&server),
            ))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn send_message_to_channel(
        &self,
        request: Request<ChannelMessage>,
    ) -> Result<Response<u32>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(_server) = server.server.upgrade() {
            Err(Status::unimplemented("Not implemented"))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn unsubscribe_from_channel(
        &self,
        request: Request<String>,
    ) -> Result<Response<()>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(_server) = server.server.upgrade() {
            Err(Status::unimplemented("Not implemented"))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    type SubscribeToChannelStream = ReceiverStream<Result<ChannelMessage, Status>>;

    async fn subscribe_to_channel(
        &self,
        request: Request<String>,
    ) -> Result<Response<Self::SubscribeToChannelStream>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(_server) = server.server.upgrade() {
            let (mut _transfer, receiver) = channel(4);
            Ok(Response::new(ReceiverStream::new(receiver)))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }
}
