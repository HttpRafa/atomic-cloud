use proto::server_service_server::ServerService;
use std::sync::Arc;
use tonic::async_trait;

use crate::controller::Controller;

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("server");
}

pub struct ServerServiceImpl {
    pub controller: Arc<Controller>,
}

#[async_trait]
impl ServerService for ServerServiceImpl {}
