use anyhow::Result;
use loading::Loading;
use simplelog::debug;

use crate::application::{
    menu::{MenuResult, MenuUtils}, network::{proto::{server, user}, EstablishedConnection}, profile::{Profile, Profiles}
};

pub struct TransferUserMenu;

// TODO: Maybe dont request everything at once, but only what is needed
struct Data {
    users: Vec<user::Item>,
    servers: Vec<server::Short>,
    groups: Vec<String>,
}

impl TransferUserMenu {
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
                            request.user_uuids.len(),
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
                                MenuResult::Failed
                            }
                        }
                    }
                    Err(error) => {
                        debug!("{}", error);
                        MenuResult::Failed
                    }
                }
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<Data> {
        let users = connection.client.get_users().await?;
        let units = connection.client.get_units().await?;
        let deployments = connection.client.get_deployments().await?;
        Ok(Data {
            users,
            units,
            deployments,
        })
    }

    fn collect_transfer_request(data: &Data) -> Result<TransferUsersRequest> {
        let users =
            MenuUtils::multi_select_no_help("Select the users to transfer", data.users.clone())?;
        let target = Self::collect_transfer_target(data)?;

        Ok(TransferUsersRequest {
            user_uuids: users.iter().map(|user| user.uuid.clone()).collect(),
            target: Some(target),
        })
    }

    fn collect_transfer_target(data: &Data) -> Result<TransferTargetValue> {
        match MenuUtils::select_no_help(
            "Select the target type",
            vec![
                TargetType::Unit,
                TargetType::Deployment,
                TargetType::Fallback,
            ],
        )? {
            TargetType::Unit => {
                let unit = MenuUtils::select_no_help(
                    "Select the unit to transfer the user to",
                    data.units.clone(),
                )?;
                Ok(TransferTargetValue {
                    target_type: TargetType::Unit as i32,
                    target: Some(unit.uuid),
                })
            }
            TargetType::Deployment => {
                let deployment = MenuUtils::select_no_help(
                    "Select the deployment to transfer the user to",
                    data.deployments.clone(),
                )?;
                Ok(TransferTargetValue {
                    target_type: TargetType::Deployment as i32,
                    target: Some(deployment),
                })
            }
            TargetType::Fallback => Ok(TransferTargetValue {
                target_type: TargetType::Fallback as i32,
                target: None,
            }),
        }
    }
}
