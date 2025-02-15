use crate::generated::exports::plugin::system::bridge::{
    Address, Capabilities, ErrorMessage, GuestGenericNode, ScopedErrors, ScreenType, Server,
    ServerProposal,
};

pub mod screen;
pub mod server;

pub struct Node {}

impl GuestGenericNode for Node {
    fn new(
        _: String,
        name: String,
        id: Option<u32>,
        capabilities: Capabilities,
        controller: String,
    ) -> Self {
        todo!()
    }

    fn tick(&self) -> Result<(), ScopedErrors> {
        todo!()
    }

    fn allocate(&self, server: ServerProposal) -> Result<Vec<Address>, ErrorMessage> {
        todo!()
    }

    fn free(&self, addresses: Vec<Address>) {
        todo!()
    }

    fn start(&self, server: Server) -> ScreenType {
        todo!()
    }

    fn restart(&self, server: Server) {
        todo!()
    }

    fn stop(&self, server: Server) {
        todo!()
    }
}
