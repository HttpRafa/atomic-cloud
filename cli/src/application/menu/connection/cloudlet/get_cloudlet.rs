use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{proto::cloudlet_management::CloudletValue, EstablishedConnection},
    profile::{Profile, Profiles},
};

pub struct GetCloudletMenu;

impl GetCloudletMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving available cloudlets from controller \"{}\"...",
            profile.name
        ));

        match connection.client.get_cloudlets().await {
            Ok(cloudlets) => {
                progress.success("Cloudlet data retrieved successfully 👍");
                progress.end();
                match MenuUtils::select_no_help(
                    "Select a cloudlet to view more details:",
                    cloudlets,
                ) {
                    Ok(cloudlet) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Fetching details for cloudlet \"{}\" from controller \"{}\"...",
                            cloudlet, profile.name
                        ));

                        match connection.client.get_cloudlet(&cloudlet).await {
                            Ok(details) => {
                                progress.success("Cloudlet details retrieved successfully 👍");
                                progress.end();
                                Self::display_details(&details);
                                MenuResult::Success
                            }
                            Err(err) => {
                                progress.fail(format!("Failed to fetch cloudlet details: {}", err));
                                progress.end();
                                MenuResult::Failed
                            }
                        }
                    }
                    Err(_) => MenuResult::Aborted,
                }
            }
            Err(err) => {
                progress.fail(format!("Failed to retrieve cloudlet data: {}", err));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    fn display_details(cloudlet: &CloudletValue) {
        info!("   <blue>🖥  <b>Cloudlet Information</>");
        info!("      <green><b>Name</>: {}", cloudlet.name);
        info!("      <green><b>Driver</>: {}", cloudlet.driver);
        if let Some(memory) = &cloudlet.memory {
            info!("      <green><b>Memory</>: {} MiB", memory);
        }
        if let Some(max_allocations) = &cloudlet.max_allocations {
            info!(
                "      <green><b>Max Allocations</>: {} Units",
                max_allocations
            );
        }
        if let Some(child) = &cloudlet.child {
            info!("      <green><b>Child Node</>: {}", child);
        }
        info!(
            "      <green><b>Controller Address</>: {}",
            cloudlet.controller_address
        );
    }
}
