use crate::{
    application::{
        auth::AuthUnitHandle,
        event::{channel::ChannelMessageSended, transfer::UserTransferRequested, EventKey},
        user::{transfer::TransferTarget, CurrentUnit},
        ControllerHandle,
    },
    VERSION,
};

use super::stream::StdReceiverStream;
use proto::unit_service_server::UnitService;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use std::{
    ops::Deref,
    str::FromStr,
    sync::{mpsc::channel, Arc},
};

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("unit");
}

pub struct UnitServiceImpl {
    pub controller: ControllerHandle,
}

#[async_trait]
impl UnitService for UnitServiceImpl {
    async fn beat_heart(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        self.controller
            .get_units()
            .handle_heart_beat(&requesting_unit);
        Ok(Response::new(()))
    }

    async fn mark_ready(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        self.controller.get_units().mark_ready(&requesting_unit);
        Ok(Response::new(()))
    }

    async fn mark_not_ready(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        self.controller
            .get_units()
            .mark_not_ready(&requesting_unit);
        Ok(Response::new(()))
    }

    async fn mark_running(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        self.controller
            .get_units()
            .mark_running(&requesting_unit);
        Ok(Response::new(()))
    }

    async fn request_stop(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        self.controller
            .get_units()
            .checked_unit_stop(&requesting_unit);
        Ok(Response::new(()))
    }

    async fn user_connected(
        &self,
        request: Request<proto::user_management::UserConnectedRequest>,
    ) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        let user = request.into_inner();
        self.controller.get_users().handle_user_connected(
            requesting_unit,
            user.name,
            Uuid::from_str(&user.uuid).map_err(|error| {
                Status::invalid_argument(format!("Failed to parse UUID: {}", error))
            })?,
        );
        Ok(Response::new(()))
    }

    async fn user_disconnected(
        &self,
        request: Request<proto::user_management::UserDisconnectedRequest>,
    ) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        let user = request.into_inner();
        self.controller.get_users().handle_user_disconnected(
            requesting_unit,
            Uuid::from_str(&user.uuid).map_err(|error| {
                Status::invalid_argument(format!("Failed to parse UUID: {}", error))
            })?,
        );
        Ok(Response::new(()))
    }

    type SubscribeToTransfersStream =
        StdReceiverStream<Result<proto::transfer_management::ResolvedTransferResponse, Status>>;
    async fn subscribe_to_transfers(
        &self,
        request: Request<()>,
    ) -> Result<Response<Self::SubscribeToTransfersStream>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        let (sender, receiver) = channel();
        self.controller
            .get_event_bus()
            .register_listener_under_unit(
                EventKey::Transfer(requesting_unit.uuid),
                Arc::downgrade(&requesting_unit),
                Box::new(move |event: &UserTransferRequested| {
                    let transfer = &event.transfer;
                    if let Some((user, _, to)) = transfer.get_strong() {
                        let address = to.allocation.primary_address();

                        let transfer = proto::transfer_management::ResolvedTransferResponse {
                            user_uuid: user.uuid.to_string(),
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
    }

    async fn transfer_all_users(
        &self,
        request: Request<proto::transfer_management::TransferAllUsersRequest>,
    ) -> Result<Response<u32>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        let transfer = request.into_inner();
        let target = match transfer.target {
            Some(target) => Some(
                match proto::transfer_management::transfer_target_value::TargetType::try_from(
                    target.target_type,
                ) {
                    Ok(proto::transfer_management::transfer_target_value::TargetType::Deployment) => {
                        TransferTarget::Deployment(
                            self.controller
                                .lock_deployments()
                                .find_by_name(&target.target)
                                .ok_or_else(|| Status::not_found("Deployment does not exist"))?,
                        )
                    }
                    _ => TransferTarget::Unit(
                        self.controller
                            .get_units()
                            .get_unit(Uuid::from_str(&target.target).map_err(|error| {
                                Status::invalid_argument(format!(
                                    "Failed to parse target UUID: {}",
                                    error
                                ))
                            })?)
                            .ok_or_else(|| Status::not_found("Unit does not exist"))?,
                    ),
                },
            ),
            None => None,
        };

        Ok(Response::new(
            self.controller
                .get_users()
                .transfer_all_users(&requesting_unit, target),
        ))
    }

    async fn transfer_user(
        &self,
        request: Request<proto::transfer_management::TransferUserRequest>,
    ) -> Result<Response<bool>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        let transfer = request.into_inner();
        let target = transfer
            .target
            .ok_or_else(|| Status::invalid_argument("Target must be provided"))?;

        let user_uuid = Uuid::from_str(&transfer.user_uuid).map_err(|error| {
            Status::invalid_argument(format!("Failed to parse user UUID: {}", error))
        })?;

        let user = self
            .controller
            .get_users()
            .get_user(user_uuid)
            .ok_or_else(|| Status::not_found("User is not connected to this controller"))?;

        // Check if the user is connected to the unit that requested the transfer
        if let CurrentUnit::Connected(unit) = user.unit.read().unwrap().deref() {
            if let Some(unit) = unit.upgrade() {
                if !Arc::ptr_eq(&unit, &requesting_unit) {
                    return Err(Status::permission_denied(
                        "User is not connected to the requesting unit",
                    ));
                }
            }
        } else {
            return Err(Status::permission_denied(
                "User is not connected to any unit",
            ));
        }

        let target = match proto::transfer_management::transfer_target_value::TargetType::try_from(
            target.target_type,
        ) {
            Ok(proto::transfer_management::transfer_target_value::TargetType::Deployment) => {
                TransferTarget::Deployment(
                    self.controller
                        .lock_deployments()
                        .find_by_name(&target.target)
                        .ok_or_else(|| Status::not_found("Deployment does not exist"))?,
                )
            }
            _ => TransferTarget::Unit(
                self.controller
                    .get_units()
                    .get_unit(Uuid::from_str(&target.target).map_err(|error| {
                        Status::invalid_argument(format!("Failed to parse target UUID: {}", error))
                    })?)
                    .ok_or_else(|| Status::not_found("Unit does not exist"))?,
            ),
        };

        let transfer = self
            .controller
            .get_users()
            .resolve_transfer(&user, &target)
            .ok_or_else(|| Status::not_found("Failed to resolve transfer"))?;
        Ok(Response::new(
            self.controller.get_users().transfer_user(transfer),
        ))
    }

    async fn send_message_to_channel(
        &self,
        request: Request<proto::channel_management::ChannelMessageValue>,
    ) -> Result<Response<u32>, Status> {
        let _requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        let message = request.into_inner();
        let count = self.controller.get_event_bus().dispatch(
            &EventKey::Channel(message.channel.clone()),
            &ChannelMessageSended { message },
        );
        Ok(Response::new(count))
    }

    async fn unsubscribe_from_channel(
        &self,
        request: Request<String>,
    ) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        self.controller
            .get_event_bus()
            .unregister_listener(EventKey::Channel(request.into_inner()), &requesting_unit);

        Ok(Response::new(()))
    }

    type SubscribeToChannelStream =
        StdReceiverStream<Result<proto::channel_management::ChannelMessageValue, Status>>;
    async fn subscribe_to_channel(
        &self,
        request: Request<String>,
    ) -> Result<Response<Self::SubscribeToChannelStream>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        let channel_name = &request.into_inner();

        let (sender, receiver) = channel();
        self.controller
            .get_event_bus()
            .register_listener_under_unit(
                EventKey::Channel(channel_name.clone()),
                Arc::downgrade(&requesting_unit),
                Box::new(move |event: &ChannelMessageSended| {
                    sender
                        .send(Ok(event.message.clone()))
                        .expect("Failed to send message to channel stream");
                }),
            );

        Ok(Response::new(StdReceiverStream::new(receiver)))
    }

    async fn reset(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let requesting_unit = request
            .extensions()
            .get::<AuthUnitHandle>()
            .expect("Failed to get unit from extensions. Is tonic broken?")
            .unit
            .upgrade()
            .ok_or_else(|| Status::not_found("The authenticated unit does not exist"))?;

        self.controller
            .get_event_bus()
            .cleanup_unit(&requesting_unit);

        Ok(Response::new(()))
    }

    async fn get_controller_version(
        &self,
        _request: Request<()>,
    ) -> Result<Response<String>, Status> {
        Ok(Response::new(VERSION.to_string()))
    }
}