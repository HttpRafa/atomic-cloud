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

        let to = match target {
            TransferTarget::Server(to) => {
                servers.get_server(&to).ok_or(ResolveError::ServerNotFound)?
                
            },
            TransferTarget::Group(group) => {
                let group = groups.get_group(&group).ok_or(ResolveError::GroupNotFound)?;
                group.find_free_server(servers).ok_or(ResolveError::NotServerAvailable)?
            },
            TransferTarget::Fallback => {
                servers.find_fallback_server(from.uuid()).ok_or(ResolveError::NotServerAvailable)?
            },
        };

        Ok(Transfer::new(user.id.clone(), from.clone(), to.id().clone()))
    }
}

pub enum ResolveError {
    UserNotFound,
    ServerNotFound,
    NotServerAvailable,
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