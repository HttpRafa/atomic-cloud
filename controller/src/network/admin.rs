use proto::admin_service_server::AdminService;
use tonic::{async_trait, Request, Response, Status};

use crate::controller::{group::ScalingPolicy, node::{Capabilities, RemoteController}, server::{Deployment, FallbackPolicy, KeyValue, Resources, Retention}, ControllerHandle, CreationResult};

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

    async fn create_node(&self, request: Request<proto::Node>) -> Result<Response<()>, Status> {
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

        let mut nodes = self.controller.lock_nodes();
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

    async fn delete_node(&self, request: Request<String>) -> Result<Response<()>, Status> {
        let mut handle = self.controller.lock_nodes();
        let node = handle.find_by_name(&request.into_inner()).ok_or(Status::not_found("Node not found"))?;

        match handle.delete_node(&node) {
            Ok(()) => Ok(Response::new(())),
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn get_node(&self, request: Request<String>) -> Result<Response<proto::Node>, Status> {
        let handle = self.controller.lock_nodes();
        let node = handle.find_by_name(&request.into_inner()).ok_or(Status::not_found("Node not found"))?;

        Ok(Response::new(proto::Node {
            name: node.name.to_owned(),
            driver: node.driver.name().to_owned(),
            memory: node.capabilities.memory,
            max_allocations: node.capabilities.max_allocations,
            sub_node: node.capabilities.sub_node.clone(),
            controller_address: node.controller.address.to_string(),
        }))
    }

    async fn get_nodes(&self, _request: Request<()>) -> Result<Response<proto::NodeList>, Status> {
        let handle = self.controller.lock_nodes();
        let mut nodes = Vec::with_capacity(handle.get_amount());
        for node in handle.get_nodes() {
            nodes.push(node.name.clone());
        }

        Ok(Response::new(proto::NodeList { nodes }))
    }

    async fn create_group(&self, request: Request<proto::Group>) -> Result<Response<()>, Status> {
        let group = request.into_inner();
        let name = &group.name;

        /* Scaling */
        let scaling = match &group.scaling {
            Some(scaling) => ScalingPolicy {
                minimum: scaling.minimum,
                maximum: scaling.maximum,
                priority: scaling.priority,
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
                deployment.disk_retention = match value {
                    x if x == Retention::Permanent as i32 => Retention::Permanent,
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

        let mut groups = self.controller.lock_groups();
        match groups.create_group(name, node_handles, scaling, resources, deployment) {
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

    async fn delete_group(&self, _request: Request<String>) -> Result<Response<()>, Status> {
        // TODO: Implement
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_group(&self, request: Request<String>) -> Result<Response<proto::Group>, Status> {
        let handle = self.controller.lock_groups();
        let group = handle.find_by_name(&request.into_inner()).ok_or(Status::not_found("Group not found"))?;

        Ok(Response::new(proto::Group {
            name: group.name.to_owned(),
            nodes: group.nodes.iter().filter_map(|node| node.upgrade().map(|node| node.name.clone())).collect(),
            scaling: Some(proto::group::Scaling {
                minimum: group.scaling.minimum,
                maximum: group.scaling.maximum,
                priority: group.scaling.priority,
            }),
            resources: Some(proto::group::Resources {
                memory: group.resources.memory,
                swap: group.resources.swap,
                cpu: group.resources.cpu,
                io: group.resources.io,
                disk: group.resources.disk,
                addresses: group.resources.addresses,
            }),
            deployment: Some(proto::group::Deployment {
                image: group.deployment.image.clone(),
                settings: group.deployment.settings.iter().map(|setting| proto::group::deployment::KeyValue {
                    key: setting.key.clone(),
                    value: setting.value.clone(),
                }).collect(),
                environment: group.deployment.environment.iter().map(|setting| proto::group::deployment::KeyValue {
                    key: setting.key.clone(),
                    value: setting.value.clone(),
                }).collect(),
                disk_retention: Some(group.deployment.disk_retention.clone() as i32),
                fallback: Some(proto::group::deployment::Fallback {
                    enabled: group.deployment.fallback.enabled,
                    priority: group.deployment.fallback.priority,
                }),
            }),
        }))
    }

    async fn get_groups(&self, _request: Request<()>) -> Result<Response<proto::GroupList>, Status> {
        let handle = self.controller.lock_groups();
        let mut groups = Vec::with_capacity(handle.get_amount());
        for name in handle.get_groups().keys() {
            groups.push(name.clone());
        }

        Ok(Response::new(proto::GroupList { groups }))
    }
}
