use anyhow::Result;
use colored::Colorize;
use log::{error, info};
use std::sync::Arc;
use tokio::{
    sync::watch::{self, Receiver, Sender},
    task::JoinHandle,
};
use tonic::transport::Server;

use crate::controller::{Controller, WeakControllerHandle};
use admin::{proto::admin_service_server::AdminServiceServer, AdminServiceImpl};
use server::{proto::server_service_server::ServerServiceServer, ServerServiceImpl};

mod admin;
mod server;

pub struct NetworkStack {
    shutdown: Sender<bool>,
    handle: JoinHandle<()>,
    controller: WeakControllerHandle,
}

impl NetworkStack {
    pub fn start(controller: Arc<Controller>) -> Self {
        info!("Starting networking stack...");

        let (sender, receiver) = watch::channel(false);

        return NetworkStack {
            shutdown: sender,
            handle: controller
                .get_runtime()
                .spawn(launch_server(controller.clone(), receiver)),
            controller: Arc::downgrade(&controller),
        };

        async fn launch_server(controller: Arc<Controller>, shutdown: Receiver<bool>) {
            if let Err(error) = run(controller, shutdown).await {
                error!("Failed to start gRPC server: {}", error);
            }
        }

        async fn run(controller: Arc<Controller>, mut shutdown: Receiver<bool>) -> Result<()> {
            let address = controller
                .configuration
                .listener
                .expect("No listener address found in the config");

            let admin_service = AdminServiceImpl {
                controller: Arc::clone(&controller),
            };
            let server_service = ServerServiceImpl { controller };

            info!(
                "Controller {} on {}",
                "listening".blue(),
                format!("{}", address).blue()
            );

            Server::builder()
                .add_service(AdminServiceServer::new(admin_service))
                .add_service(ServerServiceServer::new(server_service))
                .serve_with_shutdown(address, async {
                    shutdown.changed().await.ok();
                })
                .await?;

            Ok(())
        }
    }

    pub fn shutdown(self) {
        self.shutdown
            .send(true)
            .expect("Failed to send shutdown signal");
        if let Some(controller) = self.controller.upgrade() {
            controller
                .get_runtime()
                .block_on(self.handle)
                .expect("Failed to shutdown network stack");
        }
    }
}
