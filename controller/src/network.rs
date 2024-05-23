use std::net::SocketAddr;
use log::info;
use tonic::transport::Server;
use crate::network::controller_service_server::ControllerServiceServer;
use crate::network::server::ControllerImpl;
use tokio::task;
use crate::config::Config;

mod server;

tonic::include_proto!("control");

pub fn start_controller_server(config: &Config) {
    // This should never return None
    let address = config.listener.to_owned().expect("No listener address found in the config");
    task::spawn(async move {
        if let Err(e) = run_controller_server(address).await {
            log::error!("Failed to start gRPC server: {}", e);
        }
    });
}

async fn run_controller_server(address: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let controller = ControllerImpl {};

    info!("Controller listening on {}", address);

    Server::builder()
        .add_service(ControllerServiceServer::new(controller))
        .serve(address)
        .await?;

    Ok(())
}
