use std::{cell::RefCell, rc::Rc};

use allocation::manager::AllocationManager;
use backend::Backend;
use server::manager::ServerManager;

use crate::{
    generated::{
        exports::plugin::system::{
            bridge::{
                Address, Capabilities, ErrorMessage, Guard, GuestNode, Server, ServerProposal,
            },
            screen::ScreenType,
        },
        plugin::system::types::ScopedErrors,
    },
    plugin::config::Config,
};

mod allocation;
pub mod backend;
pub mod screen;
mod server;

pub struct InnerNode {
    /* Cloud Identification */
    identifier: String,

    /* Node */
    name: String,
    #[allow(unused)]
    capabilities: Capabilities,
    controller: String,

    /* Shared */
    config: Rc<RefCell<Config>>,

    /* Panel */
    backend: Backend,

    /* Servers and Allocations */
    allocations: RefCell<AllocationManager>,
    servers: RefCell<ServerManager>,
}

pub struct Node(pub Rc<InnerNode>);

impl Node {
    pub fn new(
        identifier: String,
        name: String,
        capabilities: Capabilities,
        controller: String,
        config: Rc<RefCell<Config>>,
        backend: Backend,
    ) -> Self {
        Self(Rc::new(InnerNode {
            identifier,
            name,
            capabilities,
            controller,
            config,
            backend,
            allocations: AllocationManager::init(),
            servers: ServerManager::init(),
        }))
    }
}

impl GuestNode for Node {
    fn tick(&self) -> Result<(), ScopedErrors> {
        Ok(())
    }

    fn allocate(&self, server: ServerProposal) -> Result<Vec<Address>, ErrorMessage> {
        self.0
            .allocations
            .borrow_mut()
            .allocate(&self.0, server)
            .map_err(|error| error.to_string())
    }

    fn free(&self, addresses: Vec<Address>) {
        self.0.allocations.borrow_mut().free(addresses)
    }

    fn start(&self, server: Server) -> ScreenType {
        ScreenType::Unsupported
    }

    fn restart(&self, server: Server) {}

    fn stop(&self, server: Server, guard: Guard) {}
}
