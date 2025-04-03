use crate::generated::{
    exports::plugin::system::event::GuestListener,
    plugin::system::{data_types::Server, types::ErrorMessage},
};

pub struct Listener();

impl GuestListener for Listener {
    fn server_start(&self, _: Server) -> Result<(), ErrorMessage> {
        unimplemented!()
    }

    fn server_stop(&self, _: Server) -> Result<(), ErrorMessage> {
        unimplemented!()
    }
}
