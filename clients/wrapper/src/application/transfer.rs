use std::{process::exit, str::FromStr};

use simplelog::{error, info};
use tonic::Streaming;
use uuid::Uuid;

use super::{
    network::{proto::transfer_management::ResolvedTransferResponse, CloudConnectionHandle},
    process::ManagedProcess,
    user::Users,
};

pub struct Transfers {
    /* Network */
    connection: CloudConnectionHandle,

    /* Stream */
    stream: Option<Streaming<ResolvedTransferResponse>>,

    /* Transfer Command */
    command: String,
}

impl Transfers {
    pub fn from_env(connection: CloudConnectionHandle) -> Self {
        let transfer_command;

        if let Ok(value) = std::env::var("TRANSFER_COMMAND") {
            transfer_command = value;
        } else {
            error!("<red>Missing</> TRANSFER_COMMAND environment variable. Please set it to the command to execute when a transfer is received");
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
                error!("<red>Failed</> to subscribe to transfers: {}", error);
            }
        }
    }

    pub async fn tick(&mut self, process: &mut ManagedProcess, users: &Users) {
        if let Some(stream) = &mut self.stream {
            while let Ok(Some(transfer)) = stream.message().await {
                if let Ok(uuid) = Uuid::from_str(&transfer.user_uuid) {
                    if let Some(user) = users.get_user_from_uuid(uuid).await {
                        info!(
                            "Transferred user <blue>{}</> to <blue>{}</>:<blue>{}</>",
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
                        error!(
                            "Received transfer from unknown user: <blue>{}</>",
                            transfer.user_uuid
                        );
                    }
                } else {
                    error!("<red>Failed</> to parse uuid: {}", transfer.user_uuid);
                }
            }
        }
    }
}
