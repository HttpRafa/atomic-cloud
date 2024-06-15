use std::sync::Arc;
use proto::{admin_service_server::AdminService, Group, GroupList, Node, NodeList};
use tonic::{async_trait, Request, Response, Status};

use crate::controller::{group::ScalingPolicy, node::Capabilities, server::{Deployment, DriverSetting, Resources, Retention}, Controller, CreationResult};

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("admin");
}

pub struct AdminServiceImpl {
    pub controller: Arc<Controller>,
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn request_stop(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        self.controller.request_stop();
        Ok(Response::new(()))
    }

    async fn create_node(&self, request: Request<Node>) -> Result<Response<()>, Status> {
        let node = request.into_inner();
        let name = &node.name;
        let driver = &node.driver;

        let mut capabilities = Capabilities::default();
        capabilities.memory = node.memory;
        capabilities.max_allocations = node.max_allocations;
        capabilities.sub_node = node.sub_node;

        let driver = match self.controller.drivers.find_by_name(driver) {
            Some(driver) => driver,
            None => return Err(Status::invalid_argument("The driver does not exist")),
        };

        let mut nodes = self.controller.lock_nodes();
        match nodes.create_node(name, driver, capabilities) {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => Err(Status::already_exists("Node already exists")),
                CreationResult::Denied(error) => Err(Status::failed_precondition(error.to_string())),
            },
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn delete_node(&self, _request: Request<String>) -> Result<Response<()>, Status> {
        // TODO: Implement
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_node(&self, request: Request<String>) -> Result<Response<Node>, Status> {
        let handle = self.controller.lock_nodes();
        let node = match handle.find_by_name(&request.into_inner()) {
            Some(node) => node.upgrade().ok_or(Status::not_found("Node not found"))?,
            None => return Err(Status::not_found("Node not found")),
        };

        Ok(Response::new(Node {
            name: node.name.to_owned(),
            driver: node.driver.name().to_owned(),
            memory: node.capabilities.memory,
            max_allocations: node.capabilities.max_allocations,
            sub_node: node.capabilities.sub_node.clone(),
        }))
    }

    async fn get_nodes(&self, _request: Request<()>) -> Result<Response<NodeList>, Status> {
        let handle = self.controller.lock_nodes();
        let mut nodes = Vec::with_capacity(handle.get_amount());
        for node in handle.get_nodes() {
            nodes.push(node.name.clone());
        }

        Ok(Response::new(NodeList { nodes }))
    }

    async fn create_group(&self, request: Request<Group>) -> Result<Response<()>, Status> {
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
                cpu: resources.cpu,
                memory: resources.memory,
                disk: resources.disk,
                addresses: resources.addresses,
            },
            None => Resources::default(),
        };

        /* Deployment */
        let mut deployment = Deployment::default();
        if let Some(value) = group.deployment {
            deployment.driver_settings = value.driver_settings.iter().map(|setting| DriverSetting {
                key: setting.key.clone(),
                value: setting.value.clone(),
            }).collect();
            if let Some(value) = value.disk_retention {
                deployment.disk_retention = match value {
                    x if x == Retention::Keep as i32 => Retention::Keep,
                    _ => Retention::Delete,
                };
            }
        }

        /* Nodes */
        let mut node_handles = Vec::with_capacity(group.nodes.len());
        for node in &group.nodes {
            let node = match self.controller.lock_nodes().find_by_name(node) {
                Some(node) => node,
                None => return Err(Status::invalid_argument(format!("Node {} does not exist", node))),
            };
            node_handles.push(node);
        }

        let mut groups = self.controller.lock_groups();
        match groups.create_group(name, node_handles, scaling, resources, deployment) {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => Err(Status::already_exists("Group already exists")),
                CreationResult::Denied(error) => Err(Status::failed_precondition(error.to_string())),
            },
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn delete_group(&self, _request: Request<String>) -> Result<Response<()>, Status> {
        // TODO: Implement
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_group(&self, _request: Request<String>) -> Result<Response<Group>, Status> {
        // TODO: Implement
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_groups(&self, _request: Request<()>) -> Result<Response<GroupList>, Status> {
        // TODO: Implement
        Err(Status::unimplemented("Not implemented"))
    }
}