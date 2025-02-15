use std::{cell::RefCell, collections::HashMap};

use anyhow::Result;

use crate::{
    error,
    generated::{
        exports::plugin::system::{
            bridge::{self, ScopedErrors, Uuid},
            screen::ScreenType,
        },
        plugin::system::types::ScopedError,
    },
    info,
    node::InnerNode,
};

use super::Server;

pub struct ServerManager {
    servers: HashMap<Uuid, Server>,
}

impl ServerManager {
    pub fn init() -> RefCell<Self> {
        RefCell::new(Self {
            servers: HashMap::new(),
        })
    }

    pub fn tick(&mut self) -> Result<(), ScopedErrors> {
        let mut errors = vec![];
        for server in self.servers.values_mut() {
            if let Err(error) = server.tick() {
                errors.push(ScopedError {
                    scope: server.name.get_name().to_string(),
                    message: error.to_string(),
                });
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(())
    }

    pub fn spawn(&mut self, node: &InnerNode, request: bridge::Server) -> ScreenType {
        let templates = node.templates.borrow();
        let name = request.name.clone();

        let Some(template) = templates.get_template(&request.allocation.spec.image) else {
            error!(
                "Template not found while starting server {}: {}",
                request.name, request.allocation.spec.image
            );
            return ScreenType::Unsupported;
        };

        let server = match Server::spawn(node, request, template) {
            Ok(server) => server,
            Err(error) => {
                error!("Failed to spawn server {}: {}", name, error);
                return ScreenType::Unsupported;
            }
        };
        let screen = server.screen();

        info!("Server {} started", name);
        self.servers.insert(name, server);
        screen
    }

    pub fn restart(&mut self, node: &InnerNode, server: bridge::Server) {
        let server = match self.servers.get_mut(&server.name) {
            Some(server) => server,
            None => {
                error!("Server not found while restarting server {}", server.name);
                return;
            }
        };

        if let Err(error) = server.restart(node) {
            error!(
                "Failed to restart server {}: {}",
                server.name.get_name(),
                error
            );
        }
    }

    pub fn stop(&mut self, node: &InnerNode, server: bridge::Server) {
        let server = match self.servers.get_mut(&server.name) {
            Some(server) => server,
            None => {
                error!("Server not found while stopping server {}", server.name);
                return;
            }
        };

        if let Err(error) = server.stop(node) {
            error!(
                "Failed to stop server {}: {}",
                server.name.get_name(),
                error
            );
        }
    }
}
