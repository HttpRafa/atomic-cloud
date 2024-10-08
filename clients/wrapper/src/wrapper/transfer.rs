use std::{process::exit, str::FromStr};

use tonic::Streaming;
use uuid::Uuid;

use super::{
    network::{proto::ResolvedTransfer, CloudConnectionHandle},
    process::ManagedProcess,
    user::Users,
};
use log::{error, info};

pub struct Transfers {
    /* Network */
    connection: CloudConnectionHandle,

    /* Stream */
    stream: Option<Streaming<ResolvedTransfer>>,

    /* Transfer Command */
    command: String,
}

impl Transfers {
    pub fn from_env(connection: CloudConnectionHandle) -> Self {
        let transfer_command;

        if let Ok(value) = std::env::var("TRANSFER_COMMAND") {
            transfer_command = value;
        } else {
            error!("Missing TRANSFER_COMMAND environment variable. Please set it to the command to execute when a transfer is received");
            exit(1);
        }

        Self::new(connection, transfer_command)
    }

    pub fn new(connection: CloudConnectionHandle, command: String) -> Self {
        Self {
            connection,
            stream: None,
            command,
        }
    }

    pub async fn subscribe(&mut self) {
        match self.connection.subscribe_to_transfers().await {
            Ok(stream) => {
                self.stream = Some(stream.into_inner());
            }
            Err(error) => {
                error!("Failed to subscribe to transfers: {}", error);
            }
        }
    }

    pub async fn tick(&mut self, process: &mut ManagedProcess, users: &Users) {
        if let Some(stream) = &mut self.stream {
            while let Ok(Some(transfer)) = stream.message().await {
                if let Some(user) = transfer.user {
                    if let Ok(uuid) = Uuid::from_str(&user.uuid) {
                        if let Some(user) = users.get_user_from_uuid(uuid).await {
                            info!(
                                "Transferred user {} to {}:{}",
                                user.name, transfer.host, transfer.port
                            );
                            let command = self
                                .command
                                .replace("%NAME%", &user.name)
                                .replace("%UUID%", &user.uuid.to_string())
                                .replace("%HOST%", &transfer.host)
                                .replace("%PORT%", &transfer.port.to_string());
                            process.write_to_stdin(&command).await;
                        } else {
                            error!("Received transfer from unknown user: {}", user.uuid);
                        }
                    } else {
                        error!("Failed to parse uuid: {}", user.uuid);
                    }
                }
            }
        }
    }
}
