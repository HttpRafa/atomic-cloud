use proto::{admin_service_server::AdminService, Node, NodeList, NodeResponse};
use tokio::sync::mpsc::Sender;
use tonic::{async_trait, Request, Response, Result, Status};

use super::NetworkTask;

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("admin");
}

pub struct AdminServiceImpl {
    pub sender: Sender<NetworkTask>
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn create_node(&self, _request: Request<Node>) -> Result<Response<NodeResponse>, Status> {
        Ok(Response::new(NodeResponse {
            success: false, message: "Not implemented".to_string()
        }))
    }
    async fn delete_node(&self, _request: Request<Node>) -> Result<Response<NodeResponse>, Status> {
        Ok(Response::new(NodeResponse {
            success: false, message: "Not implemented".to_string()
        }))
    }
    async fn get_nodes(&self, _request: Request<()>) -> Result<Response<NodeList>, Status> {
        Ok(Response::new(NodeList { 

        }))
    }
}