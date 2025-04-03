use crate::generated::{
    exports::plugin::system::{
        bridge::{Address, ErrorMessage, Guard, GuestNode, Server, ServerProposal},
        screen::ScreenType,
    },
    plugin::system::types::ScopedErrors,
};

pub struct Node();

impl GuestNode for Node {
    fn tick(&self) -> Result<(), ScopedErrors> {
        unimplemented!()
    }

    fn allocate(&self, _: ServerProposal) -> Result<Vec<Address>, ErrorMessage> {
        unimplemented!()
    }

    fn free(&self, _: Vec<Address>) {
        unimplemented!()
    }

    fn start(&self, _: Server) -> ScreenType {
        unimplemented!()
    }

    fn restart(&self, _: Server) {
        unimplemented!()
    }

    fn stop(&self, _: Server, _: Guard) {
        unimplemented!()
    }
}
