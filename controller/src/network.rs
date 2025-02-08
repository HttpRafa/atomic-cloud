use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use auth::AuthInterceptor;
use client::ClientServiceImpl;
use common::error::FancyError;
use manage::ManageServiceImpl;
use proto::{
    client::client_service_server::ClientServiceServer,
    manage::manage_service_server::ManageServiceServer,
};
use simplelog::info;
use tokio::{
    spawn,
    sync::watch::{channel, Receiver, Sender},
    task::JoinHandle,
};
use tonic::transport::Server;

use crate::{
    application::{Shared, TaskSender},
    config::Config,
};

mod auth;
pub mod client;
pub mod manage;
mod proto;

pub struct NetworkStack {
    shutdown: Sender<bool>,
    handle: JoinHandle<()>,
}

impl NetworkStack {
    pub fn start(config: &Config, shared: &Arc<Shared>, queue: &TaskSender) -> Self {
        info!("Starting network stack...");

        let (sender, receiver) = channel(false);
        let bind = *config.network_bind();
        let shared = shared.clone();
        let queue = queue.clone();

        let task = spawn(async move {
            if let Err(error) = run(bind, shared, queue, receiver).await {
                FancyError::print_fancy(&error, false);
            }
        });

        return Self {
            shutdown: sender,
            handle: task,
        };

        async fn run(
            bind: SocketAddr,
            shared: Arc<Shared>,
            queue: TaskSender,
            mut shutdown: Receiver<bool>,
        ) -> Result<()> {
            let auth_interceptor = AuthInterceptor(shared.clone());
            info!("Controller listening on {}", bind);

            Server::builder()
                .add_service(ManageServiceServer::with_interceptor(
                    ManageServiceImpl(queue.clone(), shared.clone()),
                    auth_interceptor.clone(),
                ))
                .add_service(ClientServiceServer::with_interceptor(
                    ClientServiceImpl(queue, shared),
                    auth_interceptor,
                ))
                .serve_with_shutdown(bind, async {
                    shutdown.changed().await.ok();
                })
                .await?;

            Ok(())
        }
    }

    pub async fn shutdown(self) -> Result<()> {
        info!("Stopping network stack...");
        let _ = self.shutdown.send(true); // Ignore error if receiver is dropped
        self.handle.await?;
        Ok(())
    }
}
