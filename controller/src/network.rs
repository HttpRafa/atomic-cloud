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
    application::{auth::manager::AuthManager, TaskSender},
    config::Config,
};

mod auth;
pub mod client;
pub mod manage;
mod proto;

pub const SUBSCRIPTION_BUFFER: usize = 64;

pub struct NetworkStack {
    shutdown: Sender<bool>,
    handle: JoinHandle<()>,
}

impl NetworkStack {
    pub fn start(config: &Config, auth: &Arc<AuthManager>, queue: &TaskSender) -> Self {
        info!("Starting network stack...");

        let (sender, receiver) = channel(false);
        let bind = *config.network_bind();
        let auth = auth.clone();
        let queue = queue.clone();

        let task = spawn(async move {
            if let Err(error) = run(bind, auth, queue, receiver).await {
                FancyError::print_fancy(&error, false);
            }
        });

        return Self {
            shutdown: sender,
            handle: task,
        };

        async fn run(
            bind: SocketAddr,
            auth: Arc<AuthManager>,
            queue: TaskSender,
            mut shutdown: Receiver<bool>,
        ) -> Result<()> {
            let auth_interceptor = AuthInterceptor(auth);
            info!("Controller listening on {}", bind);

            Server::builder()
                .add_service(ManageServiceServer::with_interceptor(
                    ManageServiceImpl(queue.clone()),
                    auth_interceptor.clone(),
                ))
                .add_service(ClientServiceServer::with_interceptor(
                    ClientServiceImpl(queue),
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
