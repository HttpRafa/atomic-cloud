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
use tonic::transport::{Identity, Server, ServerTlsConfig};

use crate::{
    application::{Shared, TaskSender},
    config::Config,
};

mod auth;
pub mod client;
pub mod manage;
mod proto;
pub mod tls;

pub struct NetworkStack {
    shutdown: Sender<bool>,
    handle: JoinHandle<()>,
}

impl NetworkStack {
    pub fn start(config: &Config, shared: &Arc<Shared>, queue: &TaskSender) -> Self {
        async fn run(
            bind: SocketAddr,
            identity: Option<Identity>,
            shared: Arc<Shared>,
            queue: TaskSender,
            mut shutdown: Receiver<bool>,
        ) -> Result<()> {
            let mut builder = Server::builder();

            if let Some(identity) = identity {
                builder = builder.tls_config(ServerTlsConfig::new().identity(identity))?;
            }

            let auth_interceptor = AuthInterceptor(shared.clone());
            info!("Controller listening on {}", bind);

            builder
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

        info!("Starting network stack...");

        let (sender, receiver) = channel(false);
        let bind = *config.network_bind();
        let identity = shared
            .tls
            .tls
            .as_ref()
            .map(|(_, identity)| identity.clone());
        let shared = shared.clone();
        let queue = queue.clone();

        let task = spawn(async move {
            if let Err(error) = run(bind, identity, shared, queue, receiver).await {
                FancyError::print_fancy(&error, false);
            }
        });

        Self {
            shutdown: sender,
            handle: task,
        }
    }

    pub async fn shutdown(self) -> Result<()> {
        info!("Stopping network stack...");
        let _ = self.shutdown.send(true); // Ignore error if receiver is dropped
        info!("Waiting for network stack to stop...");
        self.handle.await?;
        Ok(())
    }
}
