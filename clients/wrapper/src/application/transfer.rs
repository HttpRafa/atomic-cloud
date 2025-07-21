use std::{process::exit, str::FromStr};

use simplelog::{error, info};
use tonic::Streaming;
use uuid::Uuid;

use super::{
    network::{CloudConnectionHandle, proto::manage::transfer::TransferRes},
    process::stdin::ManagedStdin,
    user::Users,
};

pub struct Transfers {
    /* Network */
    connection: CloudConnectionHandle,

    /* Stream */
    stream: Option<Streaming<TransferRes>>,

    /* Transfer Command */
    command: String,
}

impl Transfers {
    pub fn from_env(connection: CloudConnectionHandle) -> Self {
        let transfer_command;

        if let Ok(value) = std::env::var("TRANSFER_COMMAND") {
            transfer_command = value;
        } else {
            error!(
                "Missing TRANSFER_COMMAND environment variable. Please set it to the command to execute when a transfer is received"
            );
            exit(1);
        }

        Self::new(connection, transfer_command)
    }

    pub const fn new(connection: CloudConnectionHandle, command: String) -> Self {
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

    pub async fn next_message(&mut self) -> Option<TransferRes> {
        if let Some(stream) = &mut self.stream
            && let Ok(result) = stream.message().await
        {
            return result;
        }
        None
    }

    pub async fn handle_message(
        &self,
        message: Option<TransferRes>,
        stdin: &mut ManagedStdin,
        users: &Users,
    ) {
        if let Some(transfer) = message {
            if let Ok(uuid) = Uuid::from_str(&transfer.id) {
                if let Some(user) = users.get_user_from_uuid(uuid) {
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
                    stdin.write_line(&command).await;
                } else {
                    error!("Received transfer from unknown user: {}", transfer.id);
                }
            } else {
                error!("Failed to parse uuid: {}", transfer.id);
            }
        }
    }
}
