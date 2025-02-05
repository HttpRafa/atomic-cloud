use anyhow::Result;
use inquire::InquireError;
use loading::Loading;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::manage::{
            server,
            transfer::{self, target, TransferReq},
            user,
        },
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

pub struct TransferUsersMenu;

// TODO: Maybe dont request everything at once, but only what is needed
struct Data {
    users: Vec<user::Item>,
    servers: Vec<server::Short>,
    groups: Vec<String>,
}

impl TransferUsersMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving all required data from the controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok(data) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();

                match Self::collect_transfer_request(&data) {
                    Ok(request) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Transferring {} users to target \"{}\"...",
                            request.ids.len(),
                            request.target.as_ref().unwrap()
                        ));

                        match connection.client.transfer_users(request).await {
                            Ok(_) => {
                                progress.success("User transferred successfully 👍.");
                                progress.end();
                                MenuResult::Success
                            }
                            Err(error) => {
                                progress.fail(format!("{}", error));
                                progress.end();
                                MenuResult::Failed(error)
                            }
                        }
                    }
                    Err(error) => match error {
                        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
                            MenuResult::Aborted
                        }
                        _ => MenuResult::Failed(error.into()),
                    },
                }
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<Data> {
        let users = connection.client.get_users().await?;
        let servers = connection.client.get_servers().await?;
        let groups = connection.client.get_groups().await?;
        Ok(Data {
            users,
            servers,
            groups,
        })
    }

    fn collect_transfer_request(data: &Data) -> Result<TransferReq, InquireError> {
        let users =
            MenuUtils::multi_select_no_help("Select the users to transfer", data.users.clone())?;
        let target = Self::collect_transfer_target(data)?;

        Ok(TransferReq {
            ids: users.iter().map(|user| user.id.clone()).collect(),
            target: Some(target),
        })
    }

    fn collect_transfer_target(data: &Data) -> Result<transfer::Target, InquireError> {
        match MenuUtils::select_no_help(
            "Select the target type",
            vec![
                target::Type::Server,
                target::Type::Group,
                target::Type::Fallback,
            ],
        )? {
            target::Type::Server => {
                let server = MenuUtils::select_no_help(
                    "Select the server to transfer the user to",
                    data.servers.clone(),
                )?;
                Ok(transfer::Target {
                    r#type: target::Type::Server as i32,
                    target: Some(server.id),
                })
            }
            target::Type::Group => {
                let group = MenuUtils::select_no_help(
                    "Select the group to transfer the user to",
                    data.groups.clone(),
                )?;
                Ok(transfer::Target {
                    r#type: target::Type::Group as i32,
                    target: Some(group),
                })
            }
            target::Type::Fallback => Ok(transfer::Target {
                r#type: target::Type::Fallback as i32,
                target: None,
            }),
        }
    }
}
