use getset::Getters;
use tokio::time::Instant;

use super::server::NameAndUuid;

pub mod manager;
pub mod transfer;

#[derive(Getters)]
pub struct User {
    #[getset(get = "pub")]
    id: NameAndUuid,
    server: CurrentServer,
}

pub enum CurrentServer {
    Connected(NameAndUuid),
    Transfering((Instant, NameAndUuid)),
}
