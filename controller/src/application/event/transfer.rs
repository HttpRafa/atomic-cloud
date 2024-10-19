use crate::application::user::transfer::Transfer;

use super::Event;

#[derive(Debug)]
pub struct UserTransferRequested {
    pub transfer: Transfer,
}

impl Event for UserTransferRequested {}
