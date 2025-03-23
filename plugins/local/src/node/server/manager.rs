use std::{cell::RefCell, collections::HashMap};

use anyhow::Result;

use crate::{
    error,
    generated::{
        exports::plugin::system::{
            bridge::{self, Guard, ScopedErrors, Uuid},
            screen::ScreenType,
        },
        plugin::system::types::ScopedError,
    },
    info,
    node::InnerNode,
    plugin::config::Config,
};

use super::{Server, State};

pub struct ServerManager {
    servers: HashMap<Uuid, Server>,
}

impl ServerManager {
    pub fn init() -> RefCell<Self> {
        RefCell::new(Self {
            servers: HashMap::new(),
        })
    }

    pub fn tick(&mut self, node: &str, config: &Config) -> Result<(), ScopedErrors> {
        let mut errors = vec![];
        self.servers.retain(|_, server| match server.tick(config) {
            Ok(State::Dead) => {
                info!("Server {} stopped.", server.name.get_name());
                if let Err(error) = server.cleanup(node) {
                    errors.push(ScopedError {
                        scope: server.name.get_name().to_string(),
                        message: error.to_string(),
                    });
                }
                false
            }
            Ok(_) => true,
            Err(error) => {
                errors.push(ScopedError {
                    scope: server.name.get_name().to_string(),
                    message: error.to_string(),
                });
                if let Err(error) = server.cleanup(node) {
                    errors.push(ScopedError {
                        scope: server.name.get_name().to_string(),
                        message: error.to_string(),
                    });
                }
                false
            }
        });
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

    pub fn stop(&mut self, node: &InnerNode, server: bridge::Server, guard: Guard) {
        let server = match self.servers.get_mut(&server.name) {
            Some(server) => server,
            None => {
                error!("Server not found while stopping server {}", server.name);
                return;
            }
        };

        if let Err(error) = server.stop(node, guard) {
            error!(
                "Failed to stop server {}: {}",
                server.name.get_name(),
                error
            );
        }
    }
}
