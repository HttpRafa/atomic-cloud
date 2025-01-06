use anyhow::{anyhow, Result};
use loading::Loading;
use simplelog::debug;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::resource_management::{ResourceCategory, ResourceStatus, SetResourceStatusRequest},
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

pub struct SetResourceStatusMenu;

// TODO: Maybe dont request everything at once, but only what is needed
struct Data {
    cloudlets: Vec<String>,
    deployments: Vec<String>,
}

impl SetResourceStatusMenu {
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

                match Self::collect_set_resource_status_request(&data) {
                    Ok(request) => {
                        let progress = Loading::default();
                        progress.text("Changing resource...");

                        match connection.client.set_resource_status(request).await {
                            Ok(_) => {
                                progress.success("Resource changed successfully 👍.");
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
        let cloudlets = connection.client.get_cloudlets().await?;
        let deployments = connection.client.get_deployments().await?;
        Ok(Data {
            cloudlets,
            deployments,
        })
    }

    fn collect_set_resource_status_request(data: &Data) -> Result<SetResourceStatusRequest> {
        let status = MenuUtils::select_no_help(
            "What is the new status of this resource?",
            vec![ResourceStatus::Active, ResourceStatus::Inactive],
        )?;
        let category = MenuUtils::select_no_help(
            "What type of resource to you want to change?",
            vec![
                ResourceCategory::Cloudlet,
                ResourceCategory::Deployment,
                ResourceCategory::Unit,
            ],
        )?;
        match category {
            ResourceCategory::Cloudlet => {
                let cloudlet = MenuUtils::select_no_help(
                    "Select the cloudlet to change",
                    data.cloudlets.clone(),
                )?;
                Ok(SetResourceStatusRequest {
                    category: category as i32,
                    id: cloudlet,
                    status: status as i32,
                })
            }
            ResourceCategory::Deployment => {
                let deployment = MenuUtils::select_no_help(
                    "Select the deployment to change",
                    data.deployments.clone(),
                )?;
                Ok(SetResourceStatusRequest {
                    category: category as i32,
                    id: deployment,
                    status: status as i32,
                })
            }
            ResourceCategory::Unit => Err(anyhow!("Not implemented yet")),
        }
    }
}
