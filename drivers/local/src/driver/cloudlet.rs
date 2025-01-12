use crate::exports::cloudlet::driver::bridge::{
    Address, Capabilities, GuestGenericCloudlet, RemoteController, Unit, UnitProposal,
};

use super::LocalCloudletWrapper;

impl GuestGenericCloudlet for LocalCloudletWrapper {
    fn new(
        _cloud_identifier: String,
        _name: String,
        _id: Option<u32>,
        _capabilities: Capabilities,
        _controller: RemoteController,
    ) -> Self {
        Self {}
    }

    /* This method expects that the Pterodactyl Allocations are only accessed by one atomic cloud instance */
    fn allocate_addresses(&self, _unit: UnitProposal) -> Result<Vec<Address>, String> {
        Ok(Vec::new())
    }

    fn deallocate_addresses(&self, _addresses: Vec<Address>) {}

    fn start_unit(&self, _unit: Unit) {}

    fn restart_unit(&self, _unit: Unit) {}

    fn stop_unit(&self, _unit: Unit) {}
}
