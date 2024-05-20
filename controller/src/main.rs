use crate::generated::controller_server::Controller;

pub mod generated {
    tonic::include_proto!("control");
}

#[derive(Default)]
pub struct ControllerImpl {}

#[tonic::async_trait]
impl Controller for ControllerImpl {

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}