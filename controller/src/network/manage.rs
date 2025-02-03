use proto::manage_service_server::ManageService;
use tonic::async_trait;

use crate::application::TaskSender;

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("manage");
}

pub struct ManageServiceImpl {
    queue: TaskSender,
}

#[async_trait]
impl ManageService for ManageServiceImpl {
    
}