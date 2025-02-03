use proto::client_service_server::ClientService;
use tonic::{async_trait, Request, Response, Status};

use crate::application::TaskSender;

mod beat;
mod ready;
mod running;
mod stop;

#[allow(clippy::all)]
pub mod proto {
    use tonic::include_proto;

    include_proto!("client");
}

pub struct ClientServiceImpl {
    queue: TaskSender,
}

#[async_trait]
impl ClientService for ClientServiceImpl {
    async fn beat(&self, _request: Request<()>) -> Result<Response<()>, Status> {
        todo!()
    }
}