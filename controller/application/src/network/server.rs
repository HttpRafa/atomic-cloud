use std::sync::Arc;

use tonic::async_trait;

use crate::controller::Controller;

use super::proto::controller_service_server::ControllerService;

pub struct ControllerImpl {
    pub controller: Arc<Controller>
}

#[async_trait]
impl ControllerService for ControllerImpl {

}