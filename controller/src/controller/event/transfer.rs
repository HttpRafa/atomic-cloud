use crate::controller::user::transfer::Transfer;

use super::Event;

#[derive(Debug)]
pub struct UserTransferRequested {
    pub transfer: Transfer,
}

impl Event for UserTransferRequested {}
