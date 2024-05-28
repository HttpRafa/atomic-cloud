use std::error::Error;
use std::net::SocketAddr;
use colored::Colorize;
use log::info;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tonic::transport::Server;
use tokio::task;

use crate::network::controller_service_server::ControllerServiceServer;
use crate::network::server::ControllerImpl;
use crate::config::Config;

mod server;

tonic::include_proto!("control");

pub fn start_controller_server(config: &Config) -> Receiver<NetworkTask> {
    info!("Starting networking stack...");

    let address = config.listener.expect("No listener address found in the config");
    let (sender, receiver) = channel(10);

    task::spawn(async move {
        if let Err(error) = run_controller_server(address, sender).await {
            log::error!("Failed to start gRPC server: {}", error);
        }
    });

    receiver
}

async fn run_controller_server(address: SocketAddr, sender: Sender<NetworkTask>) -> Result<(), Box<dyn Error>> {
    let controller = ControllerImpl { sender };

    info!("Controller {} on {}", "listening".blue(), format!("{}", address).blue());

    Server::builder()
        .add_service(ControllerServiceServer::new(controller))
        .serve(address)
        .await?;

    Ok(())
}

pub enum NetworkTask {
    // Add variants here
}