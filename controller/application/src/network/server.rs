use tokio::sync::mpsc::Sender;
use tonic::async_trait;
use crate::network::NetworkTask;

use super::proto::controller_service_server::ControllerService;

pub struct ControllerImpl {
    pub sender: Sender<NetworkTask>
}

#[async_trait]
impl ControllerService for ControllerImpl {

}