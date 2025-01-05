use anyhow::Result;
use auth::{AdminInterceptor, UnitInterceptor};
use simplelog::{error, info};
use std::sync::Arc;
use tokio::{
    sync::watch::{self, Receiver, Sender},
    task::JoinHandle,
};
use tonic::transport::Server;

use crate::application::{Controller, WeakControllerHandle};
use admin::{proto::admin_service_server::AdminServiceServer, AdminServiceImpl};
use unit::{proto::unit_service_server::UnitServiceServer, UnitServiceImpl};

mod admin;
mod auth;
mod stream;
pub mod unit;

pub struct NetworkStack {
    shutdown: Sender<bool>,
    handle: JoinHandle<()>,
    controller: WeakControllerHandle,
}

impl NetworkStack {
    pub fn start(controller: Arc<Controller>) -> Self {
        info!("<green>Starting</> networking stack...");

        let (sender, receiver) = watch::channel(false);

        return NetworkStack {
            shutdown: sender,
            handle: controller
                .get_runtime()
                .as_ref()
                .unwrap()
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
                .network
                .bind
                .expect("No bind address found in the config");

            let admin_service = AdminServiceImpl {
                controller: Arc::clone(&controller),
            };
            let unit_service = UnitServiceImpl {
                controller: Arc::clone(&controller),
            };

            info!("Controller <blue>listening</> on <blue>{}</>", address);

            Server::builder()
                .add_service(AdminServiceServer::with_interceptor(
                    admin_service,
                    AdminInterceptor {
                        controller: Arc::clone(&controller),
                    },
                ))
                .add_service(UnitServiceServer::with_interceptor(
                    unit_service,
                    UnitInterceptor { controller },
                ))
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
                .as_ref()
                .unwrap()
                .block_on(self.handle)
                .expect("Failed to shutdown network stack");
        }
    }
}
