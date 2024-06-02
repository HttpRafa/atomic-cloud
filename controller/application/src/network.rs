use std::net::SocketAddr;
use admin::{proto::admin_service_server::AdminServiceServer, AdminServiceImpl};
use anyhow::Result;
use colored::Colorize;
use log::info;
use server::{proto::server_service_server::ServerServiceServer, ServerServiceImpl};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tonic::transport::Server;

use crate::config::Config;

mod server;
mod admin;

pub fn start_controller_server(config: &Config) -> Receiver<NetworkTask> {
    info!("Starting networking stack...");

    let address = config.listener.expect("No listener address found in the config");
    let (sender, receiver) = channel(10);

    tokio::spawn(async move {
        if let Err(error) = run_controller_server(address, sender).await {
            log::error!("Failed to start gRPC server: {}", error);
        }
    });

    receiver
}

async fn run_controller_server(address: SocketAddr, sender: Sender<NetworkTask>) -> Result<()> {
    let admin_service = AdminServiceImpl { sender: sender.clone() };
    let server_service = ServerServiceImpl { sender };

    info!("Controller {} on {}", "listening".blue(), format!("{}", address).blue());

    Server::builder()
        .add_service(AdminServiceServer::new(admin_service))
        .add_service(ServerServiceServer::new(server_service))
        .serve(address)
        .await?;

    Ok(())
}


pub enum NetworkTask {
    // Add variants here
}