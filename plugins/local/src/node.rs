use std::{cell::RefCell, rc::Rc};

use common::allocator::NumberAllocator;
use server::manager::ServerManager;

use crate::{
    generated::exports::plugin::system::bridge::{
        Address, Capabilities, ErrorMessage, Guard, GuestNode, ScopedErrors, ScreenType, Server,
        ServerProposal,
    },
    plugin::config::Config,
    template::manager::TemplateManager,
};

pub mod screen;
pub mod server;

pub struct InnerNode {
    /* Node */
    name: String,
    #[allow(unused)]
    capabilities: Capabilities,
    controller: String,

    /* Shared */
    config: Rc<RefCell<Config>>,

    allocator: Rc<RefCell<NumberAllocator<u16>>>,
    templates: Rc<RefCell<TemplateManager>>,

    /* Servers */
    servers: RefCell<ServerManager>,
}
pub struct Node(pub Rc<InnerNode>);

impl Node {
    pub fn new(
        name: String,
        capabilities: Capabilities,
        controller: String,
        config: Rc<RefCell<Config>>,
        allocator: Rc<RefCell<NumberAllocator<u16>>>,
        templates: Rc<RefCell<TemplateManager>>,
    ) -> Self {
        Self(Rc::new(InnerNode {
            name,
            capabilities,
            controller,
            config,
            allocator,
            templates,
            servers: ServerManager::init(),
        }))
    }
}

impl GuestNode for Node {
    fn tick(&self) -> Result<(), ScopedErrors> {
        self.0.servers.borrow_mut().tick(&self.0.config.borrow())?;

        Ok(())
    }

    fn allocate(&self, server: ServerProposal) -> Result<Vec<Address>, ErrorMessage> {
        let required = server.resources.ports as usize;
        let config = self.0.config.borrow();
        let host = config.host();
        let mappings = config.mappings();

        let mut ports: Vec<Address> = mappings
            .get(&server.name)
            .map(|ports| {
                ports
                    .iter()
                    .take(required)
                    .map(|port| Address {
                        host: host.to_owned(),
                        port: *port,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut allocator = self.0.allocator.borrow_mut();
        for _ in ports.len()..required {
            if let Some(port) = allocator.allocate() {
                ports.push(Address {
                    host: host.to_owned(),
                    port,
                });
            } else {
                return Err("Failed to allocate ports".to_string());
            }
        }

        Ok(ports)
    }

    fn free(&self, addresses: Vec<Address>) {
        let mut allocator = self.0.allocator.borrow_mut();
        for address in addresses {
            allocator.release(address.port);
        }
    }

    fn start(&self, server: Server) -> ScreenType {
        self.0.servers.borrow_mut().spawn(&self.0, server)
    }

    fn restart(&self, server: Server) {
        self.0.servers.borrow_mut().restart(&self.0, server)
    }

    fn stop(&self, server: Server, guard: Guard) {
        self.0.servers.borrow_mut().stop(&self.0, server, guard)
    }
}
