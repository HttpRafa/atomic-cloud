use std::{cell::RefCell, collections::HashMap};

use crate::{
    error,
    generated::{
        exports::plugin::system::{
            bridge::{self, Guard, ScopedErrors},
            screen::ScreenType,
        },
        plugin::system::data_types::Uuid,
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

    pub fn tick(&mut self, node: &InnerNode, config: &Config) -> Result<(), ScopedErrors> {
        let errors = vec![];
        self.servers
            .retain(|_, server| match server.tick(node, config) {
                State::Dead => {
                    info!("Server {} stopped.", server.name.get_name());
                    server.cleanup(node);
                    false
                }
                _ => true,
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
        let screen = server.screen(node.config.borrow().url());

        info!("Server {} started", name);
        self.servers.insert(name, server);
        screen
    }

    pub fn restart(&mut self, node: &InnerNode, server: &bridge::Server) {
        let Some(server) = self.servers.get_mut(&server.name) else {
            error!("Server not found while restarting server {}", server.name);
            return;
        };

        server.restart(node);
    }

    pub fn stop(&mut self, node: &InnerNode, server: &bridge::Server, guard: Guard) {
        let Some(server) = self.servers.get_mut(&server.name) else {
            error!("Server not found while stopping server {}", server.name);
            return;
        };

        server.stop(node, guard);
    }
}
