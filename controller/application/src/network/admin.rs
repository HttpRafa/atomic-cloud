use std::sync::Arc;

use proto::{admin_service_server::AdminService, Node, NodeList};
use tonic::{async_trait, Request, Response, Status};

use crate::controller::{node::{Capability, NodeCreationResult}, Controller};

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("admin");
}

pub struct AdminServiceImpl {
    pub controller: Arc<Controller>
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn stop_service(&self, _request: Request<()>) -> Result<Response<()>, Status> {
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
                NodeCreationResult::Created => Ok(Response::new(())),
                NodeCreationResult::AlreadyExists => Err(Status::already_exists("Node already exists")),
                NodeCreationResult::Denied(error) => Err(Status::failed_precondition(error)),
            }
            Err(error) => Err(Status::internal(error.to_string())),
        }
    }
    async fn delete_node(&self, _request: Request<Node>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
    async fn get_nodes(&self, _request: Request<()>) -> Result<Response<NodeList>, Status> {
        Ok(Response::new(NodeList { 

        }))
    }
}