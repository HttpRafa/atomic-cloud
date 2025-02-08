use getset::Getters;
use simplelog::info;
use tokio::time::Instant;
use tonic::Status;
use uuid::Uuid;

use crate::application::{
    auth::Authorization,
    group::manager::GroupManager,
    server::{manager::ServerManager, NameAndUuid, Server},
};

use super::{manager::UserManager, CurrentServer, User};

impl<'a> Transfer<'a> {
    pub fn resolve(
        auth: &Authorization,
        user: &'a mut User,
        target: &TransferTarget,
        servers: &'a ServerManager,
        groups: &GroupManager,
    ) -> Result<Transfer<'a>, ResolveError> {
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
            user,
            from.clone(),
            to.id().clone(),
        ))
    }
}

impl UserManager {
    pub fn transfer_user(&mut self, transfer: &mut Transfer) -> bool {
        info!(
            "Transfering user {} from {} to server {}",
            transfer.user.id, transfer.from, transfer.to
        );

        
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

pub struct Transfer<'a> {
    user: &'a mut User,
    from: NameAndUuid,
    to: NameAndUuid,
}

impl<'a> Transfer<'a> {
    pub fn new(user: &'a mut User, from: NameAndUuid, to: NameAndUuid) -> Self {
        Self {
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
