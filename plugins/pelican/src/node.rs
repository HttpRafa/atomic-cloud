use std::{cell::RefCell, rc::Rc};

use remote::Remote;
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

mod remote;
pub mod screen;
mod server;

pub struct InnerNode {
    /* Node */
    name: String,
    #[allow(unused)]
    capabilities: Capabilities,
    controller: String,

    /* Shared */
    config: Rc<Config>,
    remote: Remote,

    /* Servers */
    servers: RefCell<ServerManager>,
}

pub struct Node(pub Rc<InnerNode>);

impl Node {
    pub fn new(
        name: String,
        capabilities: Capabilities,
        controller: String,
        config: Rc<Config>,
        remote: Remote,
    ) -> Self {
        Self(Rc::new(InnerNode {
            name,
            capabilities,
            controller,
            config,
            remote,
            servers: ServerManager::init(),
        }))
    }
}

impl GuestNode for Node {
    fn tick(&self) -> Result<(), ScopedErrors> {
        Ok(())
    }

    fn allocate(&self, server: ServerProposal) -> Result<Vec<Address>, ErrorMessage> {
        Ok(vec![])
    }

    fn free(&self, addresses: Vec<Address>) {}

    fn start(&self, server: Server) -> ScreenType {
        ScreenType::Unsupported
    }

    fn restart(&self, server: Server) {}

    fn stop(&self, server: Server, guard: Guard) {}
}
