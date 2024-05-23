use crate::network::controller_service_server::ControllerService;

pub struct ControllerImpl {}

#[tonic::async_trait]
impl ControllerService for ControllerImpl {

}