use std::{str::FromStr, sync::atomic::Ordering};

use proto::admin_service_server::AdminService;
use tonic::{async_trait, Request, Response, Status};
use uuid::Uuid;

use crate::application::{
    group::{ScalingPolicy, StartConstraints},
    node::{Capabilities, LifecycleStatus, RemoteController},
    server::{Deployment, FallbackPolicy, KeyValue, Resources, Retention},
    user::transfer::TransferTarget,
    ControllerHandle, CreationResult,
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
            Ok(proto::resource_management::ResourceStatus::Retired) => LifecycleStatus::Retired,
            _ => return Err(Status::invalid_argument("Invalid resource status")),
        };
        match proto::resource_management::ResourceCategory::try_from(resource.category) {
            Ok(proto::resource_management::ResourceCategory::Node) => {
                let mut handle = self.controller.lock_nodes_mut();
                let node = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Node not found"))?;
                match handle.set_node_status(&node, status) {
                    Ok(()) => Ok(Response::new(())),
                    Err(error) => Err(Status::internal(error.to_string())),
                }
            }
            Ok(proto::resource_management::ResourceCategory::Group) => {
                let mut handle = self.controller.lock_groups_mut();
                let group: std::sync::Arc<crate::application::group::Group> = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Group not found"))?;
                match handle.set_group_status(&group, status) {
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
            Ok(proto::resource_management::ResourceCategory::Node) => {
                let mut handle = self.controller.lock_nodes_mut();
                let node = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Node not found"))?;
                match handle.delete_node(&node) {
                    Ok(()) => Ok(Response::new(())),
                    Err(error) => Err(Status::internal(error.to_string())),
                }
            }
            Ok(proto::resource_management::ResourceCategory::Group) => {
                let mut handle = self.controller.lock_groups_mut();
                let group: std::sync::Arc<crate::application::group::Group> = handle
                    .find_by_name(&resource.id)
                    .ok_or(Status::not_found("Group not found"))?;
                match handle.delete_group(&group) {
                    Ok(()) => Ok(Response::new(())),
                    Err(error) => Err(Status::internal(error.to_string())),
                }
            }
            Ok(proto::resource_management::ResourceCategory::Server) => {
                let uuid = Uuid::from_str(&resource.id).map_err(|error| {
                    Status::invalid_argument(format!("Failed to parse UUID of server: {}", error))
                })?;
                let servers = self.controller.get_servers();
                let server = servers
                    .get_server(uuid)
                    .ok_or(Status::not_found("Server not found"))?;
                servers.checked_stop_server(&server);
                Ok(Response::new(()))
            }
            Err(_) => Err(Status::not_found("Invalid resource category")),
        }
    }

    async fn create_node(
        &self,
        request: Request<proto::node_management::NodeValue>,
    ) -> Result<Response<()>, Status> {
        let node = request.into_inner();
        let name = &node.name;
        let driver = &node.driver;

        let capabilities = Capabilities {
            memory: node.memory,
            max_allocations: node.max_allocations,
            sub_node: node.sub_node,
        };

        let controller = RemoteController {
            address: node.controller_address.parse().map_err(|_| {
                Status::invalid_argument("The controller address is not a valid URL")
            })?,
        };

        let driver = match self.controller.drivers.find_by_name(driver) {
            Some(driver) => driver,
            None => return Err(Status::invalid_argument("The driver does not exist")),
        };

        let mut nodes = self.controller.lock_nodes_mut();
        match nodes.create_node(name, driver, capabilities, controller) {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => Err(Status::already_exists("Node already exists")),
                CreationResult::Denied(error) => {
                    Err(Status::failed_precondition(error.to_string()))
                }
            },
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn get_node(
        &self,
        request: Request<String>,
    ) -> Result<Response<proto::node_management::NodeValue>, Status> {
        let handle = self.controller.lock_nodes();
        let node = handle
            .find_by_name(&request.into_inner())
            .ok_or(Status::not_found("Node not found"))?;

        Ok(Response::new(proto::node_management::NodeValue {
            name: node.name.to_owned(),
            driver: node.driver.name().to_owned(),
            memory: node.capabilities.memory,
            max_allocations: node.capabilities.max_allocations,
            sub_node: node.capabilities.sub_node.clone(),
            controller_address: node.controller.address.to_string(),
        }))
    }

    async fn get_nodes(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::node_management::NodeListResponse>, Status> {
        let handle = self.controller.lock_nodes();
        let mut nodes = Vec::with_capacity(handle.get_amount());
        for node in handle.get_nodes() {
            nodes.push(node.name.clone());
        }

        Ok(Response::new(proto::node_management::NodeListResponse {
            nodes,
        }))
    }

    async fn create_group(
        &self,
        request: Request<proto::group_management::GroupValue>,
    ) -> Result<Response<()>, Status> {
        let group = request.into_inner();
        let name = &group.name;

        /* Constraints */
        let constraints = match &group.constraints {
            Some(constraints) => StartConstraints {
                minimum: constraints.minimum,
                maximum: constraints.maximum,
                priority: constraints.priority,
            },
            None => StartConstraints::default(),
        };

        /* Scaling */
        let scaling = match &group.scaling {
            Some(scaling) => ScalingPolicy {
                enabled: true,
                max_players: scaling.max_players,
                start_threshold: scaling.start_threshold,
                stop_empty_servers: scaling.stop_empty_servers,
            },
            None => ScalingPolicy::default(),
        };

        /* Resources */
        let resources = match &group.resources {
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

        /* Deployment */
        let mut deployment = Deployment::default();
        if let Some(value) = group.deployment {
            deployment.image.clone_from(&value.image);
            deployment.settings = value
                .settings
                .iter()
                .map(|setting| KeyValue {
                    key: setting.key.clone(),
                    value: setting.value.clone(),
                })
                .collect();
            deployment.environment = value
                .environment
                .iter()
                .map(|setting| KeyValue {
                    key: setting.key.clone(),
                    value: setting.value.clone(),
                })
                .collect();
            if let Some(value) = value.disk_retention {
                deployment.disk_retention =
                    match proto::server_management::server_deployment::Retention::try_from(value) {
                        Ok(proto::server_management::server_deployment::Retention::Permanent) => {
                            Retention::Permanent
                        }
                        _ => Retention::Temporary,
                    };
            }
            if let Some(value) = value.fallback {
                deployment.fallback = FallbackPolicy {
                    enabled: value.enabled,
                    priority: value.priority,
                };
            }
        }

        /* Nodes */
        let mut node_handles = Vec::with_capacity(group.nodes.len());
        for node in &group.nodes {
            let node = match self.controller.lock_nodes().find_by_name(node) {
                Some(node) => node,
                None => {
                    return Err(Status::invalid_argument(format!(
                        "Node {} does not exist",
                        node
                    )))
                }
            };
            node_handles.push(node);
        }

        let mut groups = self.controller.lock_groups_mut();
        match groups.create_group(
            name,
            node_handles,
            constraints,
            scaling,
            resources,
            deployment,
        ) {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => {
                    Err(Status::already_exists("Group already exists"))
                }
                CreationResult::Denied(error) => {
                    Err(Status::failed_precondition(error.to_string()))
                }
            },
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn get_group(
        &self,
        request: Request<String>,
    ) -> Result<Response<proto::group_management::GroupValue>, Status> {
        let handle = self.controller.lock_groups();
        let group = handle
            .find_by_name(&request.into_inner())
            .ok_or(Status::not_found("Group not found"))?;
        let nodes = group
            .nodes
            .read()
            .unwrap()
            .iter()
            .filter_map(|node| node.upgrade().map(|node| node.name.clone()))
            .collect();

        Ok(Response::new(proto::group_management::GroupValue {
            name: group.name.to_owned(),
            nodes,
            constraints: Some(proto::group_management::group_value::Constraints {
                minimum: group.constraints.minimum,
                maximum: group.constraints.maximum,
                priority: group.constraints.priority,
            }),
            scaling: Some(proto::group_management::group_value::Scaling {
                max_players: group.scaling.max_players,
                start_threshold: group.scaling.start_threshold,
                stop_empty_servers: group.scaling.stop_empty_servers,
            }),
            resources: Some(proto::server_management::ServerResources {
                memory: group.resources.memory,
                swap: group.resources.swap,
                cpu: group.resources.cpu,
                io: group.resources.io,
                disk: group.resources.disk,
                addresses: group.resources.addresses,
            }),
            deployment: Some(proto::server_management::ServerDeployment {
                image: group.deployment.image.clone(),
                settings: group
                    .deployment
                    .settings
                    .iter()
                    .map(|setting| proto::common::KeyValue {
                        key: setting.key.clone(),
                        value: setting.value.clone(),
                    })
                    .collect(),
                environment: group
                    .deployment
                    .environment
                    .iter()
                    .map(|setting| proto::common::KeyValue {
                        key: setting.key.clone(),
                        value: setting.value.clone(),
                    })
                    .collect(),
                disk_retention: Some(group.deployment.disk_retention.clone() as i32),
                fallback: Some(proto::server_management::server_deployment::Fallback {
                    enabled: group.deployment.fallback.enabled,
                    priority: group.deployment.fallback.priority,
                }),
            }),
        }))
    }

    async fn get_groups(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::group_management::GroupListResponse>, Status> {
        let handle = self.controller.lock_groups();
        let mut groups = Vec::with_capacity(handle.get_amount());
        for name in handle.get_groups().keys() {
            groups.push(name.clone());
        }

        Ok(Response::new(proto::group_management::GroupListResponse {
            groups,
        }))
    }

    async fn get_servers(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::server_management::ServerListResponse>, Status> {
        let servers = self
            .controller
            .get_servers()
            .get_servers()
            .values()
            .filter_map(|server| {
                server
                    .node
                    .upgrade()
                    .map(|node| proto::server_management::SimpleServerValue {
                        name: server.name.to_string(),
                        uuid: server.uuid.to_string(),
                        group: server
                            .group
                            .as_ref()
                            .and_then(|g| g.group.upgrade().map(|grp| grp.name.to_string())),
                        node: node.name.to_string(),
                    })
            })
            .collect();

        Ok(Response::new(
            proto::server_management::ServerListResponse { servers },
        ))
    }

    async fn get_server(
        &self,
        request: Request<String>,
    ) -> Result<Response<proto::server_management::ServerValue>, Status> {
        let server_uuid = Uuid::from_str(&request.into_inner())
            .map_err(|e| Status::invalid_argument(format!("Invalid server UUID: {}", e)))?;

        let server = self
            .controller
            .get_servers()
            .get_server(server_uuid)
            .ok_or_else(|| Status::not_found("Server not found"))?;

        let node = server
            .node
            .upgrade()
            .ok_or_else(|| Status::internal("Node is no longer usable"))?;

        let state = (server
            .state
            .read()
            .map_err(|_| Status::internal("Failed to lock server state"))?)
        .clone() as i32;

        Ok(Response::new(proto::server_management::ServerValue {
            name: server.name.clone(),
            uuid: server.uuid.to_string(),
            group: server
                .group
                .as_ref()
                .and_then(|g| g.group.upgrade().map(|grp| grp.name.clone())),
            node: node.name.clone(),
            connected_users: server.connected_users.load(Ordering::Relaxed),
            rediness: server.rediness.load(Ordering::Relaxed),
            auth_token: server.auth.token.clone(),
            allocation: Some(proto::server_management::ServerAllocation {
                addresses: server
                    .allocation
                    .addresses
                    .iter()
                    .map(|addr| proto::common::Address {
                        ip: addr.ip().to_string(),
                        port: addr.port() as u32,
                    })
                    .collect(),
                resources: Some(proto::server_management::ServerResources {
                    memory: server.allocation.resources.memory,
                    swap: server.allocation.resources.swap,
                    cpu: server.allocation.resources.cpu,
                    io: server.allocation.resources.io,
                    disk: server.allocation.resources.disk,
                    addresses: server.allocation.resources.addresses,
                }),
                deployment: Some(proto::server_management::ServerDeployment {
                    image: server.allocation.deployment.image.clone(),
                    settings: server
                        .allocation
                        .deployment
                        .settings
                        .iter()
                        .map(|kv| proto::common::KeyValue {
                            key: kv.key.clone(),
                            value: kv.value.clone(),
                        })
                        .collect(),
                    environment: server
                        .allocation
                        .deployment
                        .environment
                        .iter()
                        .map(|kv| proto::common::KeyValue {
                            key: kv.key.clone(),
                            value: kv.value.clone(),
                        })
                        .collect(),
                    disk_retention: Some(server.allocation.deployment.disk_retention.clone() as i32),
                    fallback: Some(proto::server_management::server_deployment::Fallback {
                        enabled: server.allocation.deployment.fallback.enabled,
                        priority: server.allocation.deployment.fallback.priority,
                    }),
                }),
            }),
            state,
        }))
    }

    async fn transfer_user(
        &self,
        request: Request<proto::user_management::TransferUserRequest>,
    ) -> Result<Response<bool>, Status> {
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

        let target = match proto::user_management::transfer_target_value::TargetType::try_from(
            target.target_type,
        ) {
            Ok(proto::user_management::transfer_target_value::TargetType::Group) => {
                TransferTarget::Group(
                    self.controller
                        .lock_groups()
                        .find_by_name(&target.target)
                        .ok_or_else(|| Status::not_found("Group does not exist"))?,
                )
            }
            _ => TransferTarget::Server(
                self.controller
                    .get_servers()
                    .get_server(Uuid::from_str(&target.target).map_err(|error| {
                        Status::invalid_argument(format!("Failed to parse target UUID: {}", error))
                    })?)
                    .ok_or_else(|| Status::not_found("Server does not exist"))?,
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
}
