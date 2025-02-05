use transfer::Transfer;
use uuid::Uuid;

use super::server::NameAndUuid;

pub mod manager;
pub mod transfer;

pub struct User {
    name: String,
    uuid: Uuid,
    server: CurrentServer,
}

pub enum CurrentServer {
    Connected(NameAndUuid),
    Transfering(Transfer),
}