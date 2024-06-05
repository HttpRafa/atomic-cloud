use std::sync::Arc;
use proto::{admin_service_server::AdminService, Group, GroupList, Node, NodeList};
use tonic::{async_trait, Request, Response, Status};

use crate::controller::{group::ScalingPolicy, node::Capability, server::{DeploySetting, ServerResources}, Controller, CreationResult};

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

        let mut capabilities = Vec::new();
        if let Some(value) = node.limited_memory {
            capabilities.push(Capability::LimitedMemory(value));
        }
        if let Some(value) = node.unlimited_memory {
            capabilities.push(Capability::UnlimitedMemory(value));
        }
        if let Some(value) = node.max_servers {
            capabilities.push(Capability::MaxServers(value));
        }
        if let Some(value) = node.sub_node {
            capabilities.push(Capability::SubNode(value));
        }

        let driver = match self.controller.drivers.find_by_name(driver) {
            Some(driver) => driver,
            None => return Err(Status::invalid_argument("The driver does not exist")),
        };

        let mut nodes = self.controller.request_nodes().await;
        match nodes.create_node(name, driver, capabilities).await {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => Err(Status::already_exists("Node already exists")),
                CreationResult::Denied(error) => Err(Status::failed_precondition(error)),
            },
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }

    async fn delete_node(&self, _request: Request<String>) -> Result<Response<()>, Status> {
        // TODO: Implement
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_node(&self, request: Request<String>) -> Result<Response<Node>, Status> {
        let handle = self.controller.request_nodes().await;
        let node = match handle.find_by_name(&request.into_inner()).await {
            Some(node) => node,
            None => return Err(Status::not_found("Node not found")),
        };
        let node = node.lock().await;

        let mut limited_memory = None;
        let mut unlimited_memory = None;
        let mut max_servers = None;
        let mut sub_node = None;

        for capability in &node.capabilities {
            match capability {
                Capability::LimitedMemory(value) => limited_memory = Some(*value),
                Capability::UnlimitedMemory(value) => unlimited_memory = Some(*value),
                Capability::MaxServers(value) => max_servers = Some(*value),
                Capability::SubNode(value) => sub_node = Some(value.clone()),
            }
        }

        Ok(Response::new(Node {
            name: node.name.to_owned(),
            driver: node.driver.name().to_owned(),
            limited_memory,
            unlimited_memory,
            max_servers,
            sub_node,
        }))
    }

    async fn get_nodes(&self, _request: Request<()>) -> Result<Response<NodeList>, Status> {
        let handle = self.controller.request_nodes().await;
        let mut nodes = Vec::with_capacity(handle.get_amount());
        for node in handle.get_nodes() {
            nodes.push(node.lock().await.name.clone());
        }

        Ok(Response::new(NodeList { nodes }))
    }

    async fn create_group(&self, request: Request<Group>) -> Result<Response<()>, Status> {
        let group = request.into_inner();
        let name = &group.name;

        /* Scaling */
        let scaling = match &group.scaling {
            Some(scaling) => ScalingPolicy {
                min: scaling.min,
                max: scaling.max,
                priority: scaling.priority,
            },
            None => ScalingPolicy::default(),
        };

        /* Resources */
        let resources = match &group.resources {
            Some(resources) => ServerResources {
                cpu: resources.cpu,
                memory: resources.memory,
                disk: resources.disk,
            },
            None => ServerResources::default(),
        };

        /* Deployment */
        let mut deployment = Vec::new();
        if let Some(value) = group.deployment {
            if let Some(image) = value.image {
                deployment.push(DeploySetting::Image(image));
            }
        }

        /* Nodes */
        let mut node_handles = Vec::with_capacity(group.nodes.len());
        for node in &group.nodes {
            let node = match self.controller.request_nodes().await.find_by_name(node).await {
                Some(node) => node,
                None => return Err(Status::invalid_argument(format!("Node {} does not exist", node))),
            };
            node_handles.push(node);
        }

        let mut groups = self.controller.request_groups().await;
        match groups.create_group(name, node_handles, scaling, resources, deployment).await {
            Ok(result) => match result {
                CreationResult::Created => Ok(Response::new(())),
                CreationResult::AlreadyExists => Err(Status::already_exists("Group already exists")),
                CreationResult::Denied(error) => Err(Status::failed_precondition(error)),
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