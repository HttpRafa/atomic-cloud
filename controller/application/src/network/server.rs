use proto::server_service_server::ServerService;
use tokio::sync::mpsc::Sender;
use tonic::async_trait;

use super::NetworkTask;

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("server");
}

pub struct ServerServiceImpl {
    pub sender: Sender<NetworkTask>
}

#[async_trait]
impl ServerService for ServerServiceImpl {

}