use std::{str::FromStr, sync::atomic::Ordering};

use proto::{admin_service_server::AdminService, user_management::UserValue};
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::{
    application::{
        cloudlet::{Capabilities, LifecycleStatus, RemoteController},
        deployment::{ScalingPolicy, StartConstraints},
        unit::{FallbackPolicy, KeyValue, Resources, Retention, Spec},
        user::transfer::TransferTarget,
        ControllerHandle, CreationResult,
    },
    VERSION,
};

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("admin");
}

pub struct AdminServiceImpl {
    pub controller: ControllerHandle,
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn request_stop(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        self.controller.request_stop();
        Ok(Response::new(()))
    }

    async fn set_resource_status(
        &self,
        request: Request<proto::resource_management::SetResourceStatusRequest>,
    ) -> Result<Response<()>, Status> {
        let resource = request.into_inner();
        let status = match proto::resource_management::ResourceStatus::try_from(resource.status) {
            Ok(proto::resource_management::ResourceStatus::Active) => LifecycleStatus::Active,
            Ok(proto::resource_management::ResourceStatus::Inactive) => LifecycleStatus::Inactive,
            _ => return Err(Status::invalid_argument("Invalid resource status")),
        };
        match proto::resource_management::ResourceCategory::try_from(resource.category) {
            Ok(proto::resource_management::ResourceCategory::Cloudlet) => {
                let mut handle = self.controller.lock_cloudlets_mut();
                let cloudlet = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Cloudlet not found"))?;
                match handle.set_cloudlet_status(&cloudlet, status) {
                    Ok(()) => Ok(Response::new(())),
                    Err(error) => Err(Status::internal(error.to_string())),
                }
            }
            Ok(proto::resource_management::ResourceCategory::Deployment) => {
                let mut handle = self.controller.lock_deployments_mut();
                let deployment: std::sync::Arc<crate::application::deployment::Deployment> = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Deployment not found"))?;
                match handle.set_deployment_status(&deployment, status) {
                    Ok(()) => Ok(Response::new(())),
                    Err(error) => Err(Status::internal(error.to_string())),
                }
            }
            Err(_) => Err(Status::not_found("Invalid resource category")),
            _ => Err(Status::not_found(
                "This action is not possible with this resource category",
            )),
        }
    }

    async fn delete_resource(
        &self,
        request: Request<proto::resource_management::DeleteResourceRequest>,
    ) -> Result<Response<()>, Status> {
        let resource = request.into_inner();
        match proto::resource_management::ResourceCategory::try_from(resource.category) {
            Ok(proto::resource_management::ResourceCategory::Cloudlet) => {
                let mut handle = self.controller.lock_cloudlets_mut();
                let cloudlet = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Cloudlet not found"))?;
                match handle.delete_cloudlet(&cloudlet) {
                    Ok(()) => Ok(Response::new(())),
                    Err(error) => Err(Status::internal(error.to_string())),
                }
            }
            Ok(proto::resource_management::ResourceCategory::Deployment) => {
                let mut handle = self.controller.lock_deployments_mut();
                let deployment: std::sync::Arc<crate::application::deployment::Deployment> = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Deployment not found"))?;
                match handle.delete_deployment(&deployment) {
                    Ok(()) => Ok(Response::new(())),
                    Err(error) => Err(Status::internal(error.to_string())),
                }
            }
            Ok(proto::resource_management::ResourceCategory::Unit) => {
                let uuid = Uuid::from_str(&resource.id).map_err(|error| {
                    Status::invalid_argument(format!("Failed to parse UUID of the unit: {}", error))
                })?;
                let units = self.controller.get_units();
                let unit = units
                    .get_unit(uuid)
                    .ok_or(Status::not_found("Unit not found"))?;
                units.checked_unit_stop(&unit);
                Ok(Response::new(()))
            }
            Err(_) => Err(Status::not_found("Invalid resource category")),
        }
    }

    async fn get_drivers(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::driver_management::DriverListResponse>, Status> {
        let drivers = self
            .controller
            .get_drivers()
            .get_drivers()
            .iter()
            .map(|driver| driver.name().clone())
            .collect();

        Ok(Response::new(
            proto::driver_management::DriverListResponse { drivers },
        ))
    }

    async fn create_cloudlet(
        &self,
        request: Request<proto::cloudlet_management::CloudletValue>,
    ) -> Result<Response<()>, Status> {
        let cloudlet = request.into_inner();
        let name = &cloudlet.name;
        let driver = &cloudlet.driver;

        let capabilities = Capabilities {
            memory: cloudlet.memory,
            max_allocations: cloudlet.max_allocations,
            child: cloudlet.child,
        };

        let controller = RemoteController {
            address: cloudlet.controller_address.parse().map_err(|_| {
                Status::invalid_argument("The controller address is not a valid URL")
            })?,
        };

        let driver = match self.controller.drivers.find_by_name(driver) {
            Some(driver) => driver,
            None => return Err(Status::invalid_argument("The driver does not exist")),
        };

        let mut cloudlets = self.controller.lock_cloudlets_mut();
        match cloudlets.create_cloudlet(name, driver, capabilities, controller) {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => {
                    Err(Status::already_exists("Cloudlet already exists"))
                }
                CreationResult::Denied(error) => {
                    Err(Status::failed_precondition(error.to_string()))
                }
            },
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn get_cloudlet(
        &self,
        request: Request<String>,
    ) -> Result<Response<proto::cloudlet_management::CloudletValue>, Status> {
        let handle = self.controller.lock_cloudlets();
        let cloudlet = handle
            .find_by_name(&request.into_inner())
            .ok_or(Status::not_found("Cloudlet not found"))?;

        Ok(Response::new(proto::cloudlet_management::CloudletValue {
            name: cloudlet.name.to_owned(),
            driver: cloudlet.driver.name().to_owned(),
            memory: cloudlet.capabilities.memory,
            max_allocations: cloudlet.capabilities.max_allocations,
            child: cloudlet.capabilities.child.clone(),
            controller_address: cloudlet.controller.address.to_string(),
        }))
    }

    async fn get_cloudlets(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::cloudlet_management::CloudletListResponse>, Status> {
        let handle = self.controller.lock_cloudlets();
        let mut cloudlets = Vec::with_capacity(handle.get_amount());
        for cloudlet in handle.get_cloudlets() {
            cloudlets.push(cloudlet.name.clone());
        }

        Ok(Response::new(
            proto::cloudlet_management::CloudletListResponse { cloudlets },
        ))
    }

    async fn create_deployment(
        &self,
        request: Request<proto::deployment_management::DeploymentValue>,
    ) -> Result<Response<()>, Status> {
        let deployment = request.into_inner();
        let name = &deployment.name;

        /* Constraints */
        let constraints = match &deployment.constraints {
            Some(constraints) => StartConstraints {
                minimum: constraints.minimum,
                maximum: constraints.maximum,
                priority: constraints.priority,
            },
            None => StartConstraints::default(),
        };

        /* Scaling */
        let scaling = match &deployment.scaling {
            Some(scaling) => ScalingPolicy {
                enabled: true,
                start_threshold: scaling.start_threshold,
                stop_empty_units: scaling.stop_empty_units,
            },
            None => ScalingPolicy::default(),
        };

        /* Resources */
        let resources = match &deployment.resources {
            Some(resources) => Resources {
                memory: resources.memory,
                swap: resources.swap,
                cpu: resources.cpu,
                io: resources.io,
                disk: resources.disk,
                addresses: resources.addresses,
            },
            None => Resources::default(),
        };

        /* Spec */
        let mut spec = Spec::default();
        if let Some(value) = deployment.spec {
            spec.image.clone_from(&value.image);
            spec.max_players = value.max_players;
            spec.settings = value
                .settings
                .iter()
                .map(|setting| KeyValue {
                    key: setting.key.clone(),
                    value: setting.value.clone(),
                })
                .collect();
            spec.environment = value
                .environment
                .iter()
                .map(|setting| KeyValue {
                    key: setting.key.clone(),
                    value: setting.value.clone(),
                })
                .collect();
            if let Some(value) = value.disk_retention {
                spec.disk_retention =
                    match proto::unit_management::unit_spec::Retention::try_from(value) {
                        Ok(proto::unit_management::unit_spec::Retention::Permanent) => {
                            Retention::Permanent
                        }
                        _ => Retention::Temporary,
                    };
            }
            if let Some(value) = value.fallback {
                spec.fallback = FallbackPolicy {
                    enabled: value.enabled,
                    priority: value.priority,
                };
            }
        }

        /* Cloudlets */
        let mut cloudlet_handles = Vec::with_capacity(deployment.cloudlets.len());
        for cloudlet in &deployment.cloudlets {
            let cloudlet = match self.controller.lock_cloudlets().find_by_name(cloudlet) {
                Some(cloudlet) => cloudlet,
                None => {
                    return Err(Status::invalid_argument(format!(
                        "Cloudlet {} does not exist",
                        cloudlet
                    )))
                }
            };
            cloudlet_handles.push(cloudlet);
        }

        let mut deployments = self.controller.lock_deployments_mut();
        match deployments.create_deployment(
            name,
            cloudlet_handles,
            constraints,
            scaling,
            resources,
            spec,
        ) {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => {
                    Err(Status::already_exists("Deployment already exists"))
                }
                CreationResult::Denied(error) => {
                    Err(Status::failed_precondition(error.to_string()))
                }
            },
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn get_deployment(
        &self,
        request: Request<String>,
    ) -> Result<Response<proto::deployment_management::DeploymentValue>, Status> {
        let handle = self.controller.lock_deployments();
        let deployment = handle
            .find_by_name(&request.into_inner())
            .ok_or(Status::not_found("Deployment not found"))?;
        let cloudlets = deployment
            .cloudlets
            .read()
            .unwrap()
            .iter()
            .filter_map(|cloudlet| cloudlet.upgrade().map(|cloudlet| cloudlet.name.clone()))
            .collect();

        Ok(Response::new(
            proto::deployment_management::DeploymentValue {
                name: deployment.name.to_owned(),
                cloudlets,
                constraints: Some(
                    proto::deployment_management::deployment_value::Constraints {
                        minimum: deployment.constraints.minimum,
                        maximum: deployment.constraints.maximum,
                        priority: deployment.constraints.priority,
                    },
                ),
                scaling: Some(proto::deployment_management::deployment_value::Scaling {
                    start_threshold: deployment.scaling.start_threshold,
                    stop_empty_units: deployment.scaling.stop_empty_units,
                }),
                resources: Some(proto::unit_management::UnitResources {
                    memory: deployment.resources.memory,
                    swap: deployment.resources.swap,
                    cpu: deployment.resources.cpu,
                    io: deployment.resources.io,
                    disk: deployment.resources.disk,
                    addresses: deployment.resources.addresses,
                }),
                spec: Some(proto::unit_management::UnitSpec {
                    image: deployment.spec.image.clone(),
                    max_players: deployment.spec.max_players,
                    settings: deployment
                        .spec
                        .settings
                        .iter()
                        .map(|setting| proto::common::KeyValue {
                            key: setting.key.clone(),
                            value: setting.value.clone(),
                        })
                        .collect(),
                    environment: deployment
                        .spec
                        .environment
                        .iter()
                        .map(|setting| proto::common::KeyValue {
                            key: setting.key.clone(),
                            value: setting.value.clone(),
                        })
                        .collect(),
                    disk_retention: Some(deployment.spec.disk_retention.clone() as i32),
                    fallback: Some(proto::unit_management::unit_spec::Fallback {
                        enabled: deployment.spec.fallback.enabled,
                        priority: deployment.spec.fallback.priority,
                    }),
                }),
            },
        ))
    }

    async fn get_deployments(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::deployment_management::DeploymentListResponse>, Status> {
        let handle = self.controller.lock_deployments();
        let mut deployments = Vec::with_capacity(handle.get_amount());
        for name in handle.get_deployments().keys() {
            deployments.push(name.clone());
        }

        Ok(Response::new(
            proto::deployment_management::DeploymentListResponse { deployments },
        ))
    }

    async fn get_unit(
        &self,
        request: Request<String>,
    ) -> Result<Response<proto::unit_management::UnitValue>, Status> {
        let unit_uuid = Uuid::from_str(&request.into_inner())
            .map_err(|e| Status::invalid_argument(format!("Invalid unit UUID: {}", e)))?;

        let unit = self
            .controller
            .get_units()
            .get_unit(unit_uuid)
            .ok_or_else(|| Status::not_found("Unit not found"))?;

        let cloudlet = unit
            .cloudlet
            .upgrade()
            .ok_or_else(|| Status::internal("Cloudlet is no longer usable"))?;

        let state = (unit
            .state
            .read()
            .map_err(|_| Status::internal("Failed to lock unit state"))?)
        .clone() as i32;

        Ok(Response::new(proto::unit_management::UnitValue {
            name: unit.name.clone(),
            uuid: unit.uuid.to_string(),
            deployment: unit
                .deployment
                .as_ref()
                .and_then(|g| g.deployment.upgrade().map(|grp| grp.name.clone())),
            cloudlet: cloudlet.name.clone(),
            connected_users: unit.connected_users.load(Ordering::Relaxed),
            rediness: unit.rediness.load(Ordering::Relaxed),
            auth_token: unit.auth.token.clone(),
            allocation: Some(proto::unit_management::UnitAllocation {
                addresses: unit
                    .allocation
                    .addresses
                    .iter()
                    .map(|addr| proto::common::Address {
                        ip: addr.ip().to_string(),
                        port: addr.port() as u32,
                    })
                    .collect(),
                resources: Some(proto::unit_management::UnitResources {
                    memory: unit.allocation.resources.memory,
                    swap: unit.allocation.resources.swap,
                    cpu: unit.allocation.resources.cpu,
                    io: unit.allocation.resources.io,
                    disk: unit.allocation.resources.disk,
                    addresses: unit.allocation.resources.addresses,
                }),
                spec: Some(proto::unit_management::UnitSpec {
                    image: unit.allocation.spec.image.clone(),
                    max_players: unit.allocation.spec.max_players,
                    settings: unit
                        .allocation
                        .spec
                        .settings
                        .iter()
                        .map(|kv| proto::common::KeyValue {
                            key: kv.key.clone(),
                            value: kv.value.clone(),
                        })
                        .collect(),
                    environment: unit
                        .allocation
                        .spec
                        .environment
                        .iter()
                        .map(|kv| proto::common::KeyValue {
                            key: kv.key.clone(),
                            value: kv.value.clone(),
                        })
                        .collect(),
                    disk_retention: Some(unit.allocation.spec.disk_retention.clone() as i32),
                    fallback: Some(proto::unit_management::unit_spec::Fallback {
                        enabled: unit.allocation.spec.fallback.enabled,
                        priority: unit.allocation.spec.fallback.priority,
                    }),
                }),
            }),
            state,
        }))
    }

    async fn get_units(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::unit_management::UnitListResponse>, Status> {
        let units = self
            .controller
            .get_units()
            .get_units()
            .values()
            .filter_map(|unit| {
                unit.cloudlet
                    .upgrade()
                    .map(|cloudlet| proto::unit_management::SimpleUnitValue {
                        name: unit.name.to_string(),
                        uuid: unit.uuid.to_string(),
                        deployment: unit
                            .deployment
                            .as_ref()
                            .and_then(|d| d.deployment.upgrade().map(|d| d.name.to_string())),
                        cloudlet: cloudlet.name.to_string(),
                    })
            })
            .collect();

        Ok(Response::new(proto::unit_management::UnitListResponse {
            units,
        }))
    }

    async fn get_users(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::user_management::UserListResponse>, Status> {
        let users = self
            .controller
            .get_users()
            .get_users()
            .iter()
            .map(|user| UserValue {
                name: user.name.to_string(),
                uuid: user.uuid.to_string(),
            })
            .collect();

        Ok(Response::new(proto::user_management::UserListResponse {
            users,
        }))
    }

    async fn transfer_users(
        &self,
        request: Request<proto::transfer_management::TransferUsersRequest>,
    ) -> Result<Response<u32>, Status> {
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
                .ok_or_else(|| Status::not_found("User is not connected to this controller"))?;

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
