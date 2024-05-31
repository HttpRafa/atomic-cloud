use tokio::sync::mpsc::Sender;
use tonic::async_trait;
use crate::network::controller_service_server::ControllerService;
use crate::network::NetworkTask;

pub struct ControllerImpl {
    pub sender: Sender<NetworkTask>
}

#[async_trait]
impl ControllerService for ControllerImpl {

}