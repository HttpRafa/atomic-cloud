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

        self.controller.get_units().mark_not_ready(&requesting_unit);
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

        self.controller.get_units().mark_running(&requesting_unit);
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
                            host: address.host.clone(),
                            port: address.port as u32,
                        };
                        sender
                            .send(Ok(transfer))
                            .expect("Failed to send message to transfer stream");
                    }
                }),
            );

        Ok(Response::new(StdReceiverStream::new(receiver)))
    }

    async fn transfer_users(
        &self,
        request: Request<proto::transfer_management::TransferUsersRequest>,
    ) -> Result<Response<u32>, Status> {
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

        let target =
            match proto::transfer_management::transfer_target_value::TargetType::try_from(
                target.target_type,
            ) {
                Ok(proto::transfer_management::transfer_target_value::TargetType::Deployment) => {
                    TransferTarget::Deployment(
                        self.controller
                            .lock_deployments()
                            .find_by_name(&target.target.ok_or_else(|| {
                                Status::invalid_argument("Target must be provided")
                            })?)
                            .ok_or_else(|| Status::not_found("Deployment does not exist"))?,
                    )
                }
                Ok(proto::transfer_management::transfer_target_value::TargetType::Unit) => {
                    TransferTarget::Unit(
                        self.controller
                            .get_units()
                            .get_unit(
                                Uuid::from_str(&target.target.ok_or_else(|| {
                                    Status::invalid_argument("Target must be provided")
                                })?)
                                .map_err(|error| {
                                    Status::invalid_argument(format!(
                                        "Failed to parse target UUID: {}",
                                        error
                                    ))
                                })?,
                            )
                            .ok_or_else(|| Status::not_found("Unit does not exist"))?,
                    )
                }
                Ok(proto::transfer_management::transfer_target_value::TargetType::Fallback) => {
                    TransferTarget::Fallback
                }
                Err(error) => return Err(Status::invalid_argument(error.to_string())),
            };

        let mut count = 0;
        for user_uuid in &transfer.user_uuids {
            let user_uuid = Uuid::from_str(user_uuid).map_err(|error| {
                Status::invalid_argument(format!("Failed to parse user UUID: {}", error))
            })?;

            let user = self
                .controller
                .get_users()
                .get_user(user_uuid)
                .ok_or_else(|| {
                    Status::not_found(format!(
                        "User {} is not connected to this controller",
                        user_uuid
                    ))
                })?;

            // Check if the user is connected to the unit that requested the transfer
            if let CurrentUnit::Connected(unit) = user.unit.read().unwrap().deref() {
                if let Some(unit) = unit.upgrade() {
                    if !Arc::ptr_eq(&unit, &requesting_unit) {
                        return Err(Status::permission_denied(format!(
                            "User {} is not connected to the requesting unit",
                            user_uuid
                        )));
                    }
                }
            } else {
                return Err(Status::permission_denied(format!(
                    "User {} is not connected to the requesting unit",
                    user_uuid
                )));
            }

            let transfer = self
                .controller
                .get_users()
                .resolve_transfer(&user, &target)
                .ok_or_else(|| Status::not_found("Failed to resolve transfer"))?;

            if self.controller.get_users().transfer_user(transfer) {
                count += 1;
            }
        }

        Ok(Response::new(count))
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

    async fn get_units(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::unit_information::UnitListResponse>, Status> {
        let units = self
            .controller
            .get_units()
            .get_units()
            .values()
            .map(|unit| proto::unit_information::SimpleUnitValue {
                name: unit.name.clone(),
                uuid: unit.uuid.to_string(),
                deployment: unit
                    .deployment
                    .as_ref()
                    .and_then(|d| d.deployment.upgrade().map(|d| d.name.clone())),
            })
            .collect();

        Ok(Response::new(proto::unit_information::UnitListResponse {
            units,
        }))
    }

    async fn get_deployments(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::deployment_information::DeploymentListResponse>, Status> {
        let handle = self.controller.lock_deployments();
        let mut deployments = Vec::with_capacity(handle.get_amount());
        for name in handle.get_deployments().keys() {
            deployments.push(name.clone());
        }

        Ok(Response::new(
            proto::deployment_information::DeploymentListResponse { deployments },
        ))
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

    async fn get_protocol_version(&self, _request: Request<()>) -> Result<Response<u32>, Status> {
        Ok(Response::new(VERSION.protocol))
    }

    async fn get_controller_version(
        &self,
        _request: Request<()>,
    ) -> Result<Response<String>, Status> {
        Ok(Response::new(VERSION.to_string()))
    }
}
