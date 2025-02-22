use std::sync::Arc;

use simplelog::info;
use tokio::time::Instant;
use tonic::Status;
use uuid::Uuid;

use crate::application::{
    auth::Authorization,
    group::manager::GroupManager,
    server::{manager::ServerManager, NameAndUuid, Server},
    Shared,
};

use super::{CurrentServer, User};

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

        let CurrentServer::Connected(from) = &user.server else {
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

        Ok(Transfer::new(user, from.clone(), to, Instant::now()))
    }

    pub async fn transfer_user(
        transfer: &mut Transfer<'a>,
        shared: &Arc<Shared>,
    ) -> Result<(), Status> {
        info!(
            "Transfering user {} from {} to server {}",
            transfer.user.id,
            transfer.from,
            transfer.to.id()
        );
        if let Some(data) = transfer.to.new_transfer(transfer.user.id.uuid()) {
            shared
                .subscribers
                .publish_transfer(transfer.from.uuid(), data)
                .await;

            transfer.user.server =
                CurrentServer::Transfering((transfer.timestamp, transfer.to.id().clone()));
            Ok(())
        } else {
            Err(Status::unavailable(
                "Target server seems to have no network address",
            ))
        }
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
    to: &'a Server,
    timestamp: Instant,
}

impl<'a> Transfer<'a> {
    fn new(user: &'a mut User, from: NameAndUuid, to: &'a Server, timestamp: Instant) -> Self {
        Self {
            user,
            from,
            to,
            timestamp,
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
