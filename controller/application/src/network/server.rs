use tokio::sync::mpsc::Sender;
use crate::network::controller_service_server::ControllerService;
use crate::network::NetworkTask;

pub struct ControllerImpl {
    pub sender: Sender<NetworkTask>
}

#[tonic::async_trait]
impl ControllerService for ControllerImpl {

}