use getset::Getters;
use tokio::time::Instant;
use uuid::Uuid;

use crate::application::{group::manager::GroupManager, server::{manager::ServerManager, NameAndUuid}};

use super::{manager::UserManager, CurrentServer, User};

impl UserManager {
    pub fn resolve(&mut self, user: &User, target: TransferTarget, servers: &ServerManager, groups: &GroupManager) -> Result<Transfer, ResolveError> {
        let from = if let CurrentServer::Connected(from) = &user.server {
            from
        } else {
            return Err(ResolveError::UserNotFound);
        };

        match target {
            TransferTarget::Server(to) => {
                let to = servers.get_server(&to).ok_or(ResolveError::ServerNotFound)?;
                Ok(Transfer::new(user.id.clone(), from.clone(), to.id().clone()))
            },
            TransferTarget::Group(group) => {
                
            },
            TransferTarget::Fallback => {

            },
        }
    }
}

pub enum ResolveError {
    UserNotFound,
    ServerNotFound,
    GroupNotFound,
}

pub enum TransferTarget {
    Server(Uuid),
    Group(String),
    Fallback,
}

#[derive(Getters)]
pub struct Transfer {
    #[getset(get = "pub")]
    timestamp: Instant,
    #[getset(get = "pub")]
    user: NameAndUuid,
    #[getset(get = "pub")]
    from: NameAndUuid,
    #[getset(get = "pub")]
    to: NameAndUuid,
}

impl Transfer {
    pub fn new(user: NameAndUuid, from: NameAndUuid, to: NameAndUuid) -> Self {
        Self {
            timestamp: Instant::now(),
            user,
            from,
            to,
        }
    }
}