use anyhow::Result;
use auth::{AdminInterceptor, UnitInterceptor};
use simplelog::{error, info};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    spawn, sync::watch::{self, Receiver, Sender}, task::JoinHandle
};
use tonic::transport::Server;

use crate::application::{auth::AuthValidator, Controller,};
use admin::{proto::admin_service_server::AdminServiceServer, AdminServiceImpl};
use unit::{proto::unit_service_server::UnitServiceServer, UnitServiceImpl};

mod admin;
mod auth;
mod stream;
pub mod unit;

pub struct NetworkStack {
    shutdown: Sender<bool>,
    handle: JoinHandle<()>,
}

impl NetworkStack {
    pub fn start(controller: &Controller) -> Self {
        info!("<green>Starting</> networking stack...");

        let (sender, receiver) = watch::channel(false);
        let bind = controller.get_config().network.bind.clone();
        let validator = controller.get_auth().get_validator();

        return NetworkStack {
            shutdown: sender,
            handle: spawn(launch_server(bind, validator, receiver)),
        };

        async fn launch_server(bind: SocketAddr, validator: AuthValidator, shutdown: Receiver<bool>) {
            if let Err(error) = run(bind, validator, shutdown).await {
                error!("Failed to start gRPC server: {}", error);
            }
        }

        async fn run(bind: SocketAddr, validator: AuthValidator, mut shutdown: Receiver<bool>) -> Result<()> {
            let admin_service = AdminServiceImpl {
            };
            let unit_service = UnitServiceImpl {
            };

            info!("Controller <blue>listening</> on <blue>{}</>", bind);

            Server::builder()
                .add_service(AdminServiceServer::with_interceptor(
                    admin_service,
                    AdminInterceptor {
                        validator: validator.clone(),
                    },
                ))
                .add_service(UnitServiceServer::with_interceptor(
                    unit_service,
                    UnitInterceptor { validator },
                ))
                .serve_with_shutdown(bind, async {
                    shutdown.changed().await.ok();
                })
                .await?;

            Ok(())
        }
    }

    pub async fn shutdown(self) {
        self.shutdown
            .send(true)
            .expect("Failed to send shutdown signal");
        self.handle.await;
    }
}
