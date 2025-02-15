use crate::generated::exports::plugin::system::bridge::{
    Address, Capabilities, ErrorMessage, GuestGenericNode, ScopedErrors, ScreenType, Server,
    ServerProposal,
};

pub mod screen;
pub mod server;

pub struct Node {}

impl GuestGenericNode for Node {
    async fn new(
        _: String,
        name: String,
        id: Option<u32>,
        capabilities: Capabilities,
        controller: String,
    ) -> Self {
        todo!()
    }

    async fn tick(&self) -> Result<(), ScopedErrors> {
        todo!()
    }

    async fn allocate(&self, server: ServerProposal) -> Result<Vec<Address>, ErrorMessage> {
        todo!()
    }

    async fn free(&self, addresses: Vec<Address>) {
        todo!()
    }

    async fn start(&self, server: Server) -> ScreenType {
        todo!()
    }

    async fn restart(&self, server: Server) {
        todo!()
    }

    async fn stop(&self, server: Server) {
        todo!()
    }
}
