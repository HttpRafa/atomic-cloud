use std::sync::Arc;
use anyhow::Result;
use colored::Colorize;
use log::info;
use proto::controller_service_server::ControllerServiceServer;
use tokio::task::JoinHandle;
use tonic::transport::Server;

use crate::controller::Controller;
use crate::network::server::ControllerImpl;

mod server;

#[allow(clippy::all)]
mod proto {
    use tonic::include_proto;

    include_proto!("control");
}

pub fn start_controller_server(controller: Arc<Controller>) -> JoinHandle<()> {
    info!("Starting networking stack...");

    tokio::spawn(async move {
        if let Err(error) = run_controller_server(controller).await {
            log::error!("Failed to start gRPC server: {}", error);
        }
    })
}

async fn run_controller_server(controller: Arc<Controller>) -> Result<()> {
    let address = controller.configuration.listener.expect("No listener address found in the config");
    let controller = ControllerImpl { controller };

    info!("Controller {} on {}", "listening".blue(), format!("{}", address).blue());

    Server::builder()
        .add_service(ControllerServiceServer::new(controller))
        .serve(address)
        .await?;

    Ok(())
}