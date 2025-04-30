use crate::{generated::{
    exports::plugin::system::event::GuestListener,
    plugin::system::{data_types::Server, types::ErrorMessage},
}, info};

pub struct Listener();

impl GuestListener for Listener {
    fn server_start(&self, _: Server) -> Result<(), ErrorMessage> {
        unimplemented!()
    }

    fn server_stop(&self, server: Server) -> Result<(), ErrorMessage> {
        info!("Server {} is stopping updating dns records...", server.name);
        Ok(())
    }

    fn server_change_ready(&self, server: Server, ready: bool) -> Result<(), ErrorMessage> {
        if !ready { return Ok(()); }

        info!("Server {} is ready updating dns records...", server.name);
        Ok(())
    }
}
