use std::{cell::RefCell, collections::HashMap};


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

    pub fn start(&mut self, node: &InnerNode, request: bridge::Server) -> ScreenType {
        let name = request.name.clone();

        let server = match Server::start(node, request) {
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
}
