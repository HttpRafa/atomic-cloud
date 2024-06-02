use std::sync::Arc;
use anyhow::Result;
use log::{info, error};
use tokio::task::JoinHandle;
use tonic::transport::Server;
use colored::Colorize;

use admin::{proto::admin_service_server::AdminServiceServer, AdminServiceImpl};
use server::{proto::server_service_server::ServerServiceServer, ServerServiceImpl};
use crate::controller::Controller;

mod server;
mod admin;

pub fn start_controller_server(controller: Arc<Controller>) -> JoinHandle<()> {
    info!("Starting networking stack...");

    tokio::spawn(async move {
        if let Err(error) = run_controller_server(controller).await {
            error!("Failed to start gRPC server: {}", error);
        }
    })
}

async fn run_controller_server(controller: Arc<Controller>) -> Result<()> {
    let address = controller.configuration.listener.expect("No listener address found in the config");

    let admin_service = AdminServiceImpl { controller: Arc::clone(&controller) };
    let server_service = ServerServiceImpl { controller };

    info!("Controller {} on {}", "listening".blue(), format!("{}", address).blue());

    Server::builder()
        .add_service(AdminServiceServer::new(admin_service))
        .add_service(ServerServiceServer::new(server_service))
        .serve(address)
        .await?;

    Ok(())
}