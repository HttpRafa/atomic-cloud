use getset::Getters;
use tokio::time::Instant;
use tonic::Status;
use uuid::Uuid;

use crate::application::{
    auth::Authorization,
    group::manager::GroupManager,
    server::{manager::ServerManager, NameAndUuid},
};

use super::{CurrentServer, User};

impl Transfer {
    pub fn resolve(
        auth: &Authorization,
        user: &User,
        target: &TransferTarget,
        servers: &ServerManager,
        groups: &GroupManager,
    ) -> Result<Transfer, ResolveError> {
        // Check if auth is allowed to transfer user
        if let Some(server) = auth.get_server() {
            if let CurrentServer::Connected(current) = &user.server {
                if current.uuid() != server.uuid() {
                    return Err(ResolveError::AccessDenied);
                }
            } else {
                return Err(ResolveError::AccessDenied);
            }
        }

        let from = if let CurrentServer::Connected(from) = &user.server {
            from
        } else {
            return Err(ResolveError::UserNotFound);
        };

        let to = match target {
            TransferTarget::Server(to) => {
                servers.get_server(to).ok_or(ResolveError::ServerNotFound)?
            }
            TransferTarget::Group(group) => {
                let group = groups.get_group(group).ok_or(ResolveError::GroupNotFound)?;
                group
                    .find_free_server(servers)
                    .ok_or(ResolveError::NotServerAvailable)?
            }
            TransferTarget::Fallback => servers
                .find_fallback_server(from.uuid())
                .ok_or(ResolveError::NotServerAvailable)?,
        };

        Ok(Transfer::new(
            user.id.clone(),
            from.clone(),
            to.id().clone(),
        ))
    }
}

pub enum ResolveError {
    UserNotFound,
    ServerNotFound,
    NotServerAvailable,
    GroupNotFound,

    AccessDenied,
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

impl From<ResolveError> for Status {
    fn from(val: ResolveError) -> Self {
        match val {
            ResolveError::UserNotFound => Status::not_found("User not found"),
            ResolveError::ServerNotFound => Status::not_found("Server not found"),
            ResolveError::NotServerAvailable => Status::unavailable("Server not available"),
            ResolveError::GroupNotFound => Status::not_found("Group not found"),
            ResolveError::AccessDenied => {
                Status::permission_denied("Missing permissions to transfer user")
            }
        }
    }
}
