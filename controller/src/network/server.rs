use std::{
    str::FromStr,
    sync::{mpsc::channel, Arc},
};

use proto::{
    server_service_server::ServerService, ChannelMessage, ResolvedTransfer, Transfer,
    TransferTarget, User,
};
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::controller::{
    auth::AuthServerHandle,
    event::{channel::ChannelMessageSended, transfer::UserTransferRequested, EventKey},
    ControllerHandle,
};

use super::stream::StdReceiverStream;

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

    type SubscribeToTransfersStream = StdReceiverStream<Result<ResolvedTransfer, Status>>;
    async fn subscribe_to_transfers(
        &self,
        request: Request<()>,
    ) -> Result<Response<Self::SubscribeToTransfersStream>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            // Create a stream and register it with the user system
            let (sender, receiver) = channel();
            self.controller
                .get_event_bus()
                .register_listener_under_server(
                    EventKey::Transfer(server.uuid),
                    Arc::downgrade(&server),
                    Box::new(move |event: &UserTransferRequested| {
                        let transfer = &event.transfer;
                        if let Some((user, _, to)) = transfer.get_strong() {
                            let user = User {
                                name: user.name.clone(),
                                uuid: user.uuid.to_string(),
                            };
                            let address = to.allocation.primary_address();

                            let transfer = ResolvedTransfer {
                                user: Some(user),
                                host: address.ip().to_string(),
                                port: address.port() as u32,
                            };
                            sender
                                .send(Ok(transfer))
                                .expect("Failed to send message to transfer stream");
                        }
                    }),
                );

            Ok(Response::new(StdReceiverStream::new(receiver)))
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
                self.controller.get_users().transfer_all_users(&server),
            ))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    async fn transfer_user(&self, request: Request<Transfer>) -> Result<Response<bool>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(_server) = server.server.upgrade() {
            Err(Status::unimplemented("Not yet implemented"))
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
            let message = request.into_inner();
            let count = self.controller.get_event_bus().dispatch(
                &EventKey::Channel(message.channel.clone()),
                &ChannelMessageSended { message },
            );
            Ok(Response::new(count))
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
        if let Some(server) = server.server.upgrade() {
            // Unsubscribe from the channel
            self.controller
                .get_event_bus()
                .unregister_listener(EventKey::Channel(request.into_inner()), &server);

            Ok(Response::new(()))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }

    type SubscribeToChannelStream = StdReceiverStream<Result<ChannelMessage, Status>>;
    async fn subscribe_to_channel(
        &self,
        request: Request<String>,
    ) -> Result<Response<Self::SubscribeToChannelStream>, Status> {
        let server = request
            .extensions()
            .get::<AuthServerHandle>()
            .expect("Failed to get server from extensions. Is tonic broken?");
        if let Some(server) = server.server.upgrade() {
            let channel_name = &request.into_inner();

            // Create a stream and register it with the channel
            let (sender, receiver) = channel();
            self.controller
                .get_event_bus()
                .register_listener_under_server(
                    EventKey::Channel(channel_name.clone()),
                    Arc::downgrade(&server),
                    Box::new(move |event: &ChannelMessageSended| {
                        sender
                            .send(Ok(event.message.clone()))
                            .expect("Failed to send message to channel stream");
                    }),
                );

            Ok(Response::new(StdReceiverStream::new(receiver)))
        } else {
            Err(Status::not_found("The authenticated server does not exist"))
        }
    }
}
