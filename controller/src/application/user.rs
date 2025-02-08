use tokio::time::Instant;
use transfer::Transfer;

use super::server::NameAndUuid;

pub mod manager;
pub mod transfer;

pub struct User {
    id: NameAndUuid,
    server: CurrentServer,
}

pub enum CurrentServer {
    Connected(NameAndUuid),
    Transfering((Instant, NameAndUuid)),
}
