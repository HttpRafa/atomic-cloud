use anyhow::Result;
use loading::Loading;
use simplelog::debug;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::{
            resource_management::{DeleteResourceRequest, ResourceCategory},
            unit_management::SimpleUnitValue,
        },
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

pub struct DeleteResourceMenu;

// TODO: Maybe dont request everything at once, but only what is needed
struct Data {
    cloudlets: Vec<String>,
    deployments: Vec<String>,
    units: Vec<SimpleUnitValue>,
}

impl DeleteResourceMenu {
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

                match Self::collect_delete_resource(&data) {
                    Ok(request) => {
                        let progress = Loading::default();
                        progress.text("Deleting resource...");

                        match connection.client.delete_resource(request).await {
                            Ok(_) => {
                                progress.success("Resource deleted successfully 👍.");
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
        let units = connection.client.get_units().await?;
        Ok(Data {
            cloudlets,
            deployments,
            units,
        })
    }

    fn collect_delete_resource(data: &Data) -> Result<DeleteResourceRequest> {
        let category = MenuUtils::select_no_help(
            "What type of resource to you want to delete?",
            vec![
                ResourceCategory::Cloudlet,
                ResourceCategory::Deployment,
                ResourceCategory::Unit,
            ],
        )?;
        match category {
            ResourceCategory::Cloudlet => {
                let cloudlet = MenuUtils::select_no_help(
                    "Select the cloudlet to delete",
                    data.cloudlets.clone(),
                )?;
                Ok(DeleteResourceRequest {
                    category: category as i32,
                    id: cloudlet,
                })
            }
            ResourceCategory::Deployment => {
                let deployment = MenuUtils::select_no_help(
                    "Select the deployment to delete",
                    data.deployments.clone(),
                )?;
                Ok(DeleteResourceRequest {
                    category: category as i32,
                    id: deployment,
                })
            }
            ResourceCategory::Unit => {
                let unit =
                    MenuUtils::select_no_help("Select the unit to delete", data.units.clone())?;
                Ok(DeleteResourceRequest {
                    category: category as i32,
                    id: unit.uuid,
                })
            }
        }
    }
}
