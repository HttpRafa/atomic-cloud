use loading::Loading;
use simplelog::{info, warn};

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{proto::deployment_management::DeploymentValue, EstablishedConnection},
    profile::{Profile, Profiles},
};

pub struct GetDeploymentMenu;

impl GetDeploymentMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Fetching all available deployments from the controller \"{}\"...",
            profile.name
        ));

        match connection.client.get_deployments().await {
            Ok(deployments) => {
                progress.success("Deployment data retrieved successfully 👍");
                progress.end();

                match MenuUtils::select_no_help(
                    "Select a deployment to view more details:",
                    deployments,
                ) {
                    Ok(deployment) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Fetching details for deployment \"{}\" from controller \"{}\"...",
                            deployment, profile.name
                        ));

                        match connection.client.get_deployment(&deployment).await {
                            Ok(deployment_details) => {
                                progress.success("Deployment details retrieved successfully 👍");
                                progress.end();

                                Self::display_details(&deployment_details);
                                MenuResult::Success
                            }
                            Err(error) => {
                                progress.fail(format!("{}", error));
                                progress.end();
                                MenuResult::Failed
                            }
                        }
                    }
                    Err(_) => MenuResult::Aborted,
                }
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    fn display_details(deployment_details: &DeploymentValue) {
        info!("   <blue>🖥  <b>Deployment Details</>");
        info!("      <green><b>Name</>: {}", deployment_details.name);

        if !deployment_details.cloudlets.is_empty() {
            info!("      <green><b>Cloudlets</>:");
            for cloudlet in &deployment_details.cloudlets {
                info!("         - <green>{}</>", cloudlet);
            }
        } else {
            warn!("      <yellow><b>Cloudlets</>: None");
        }

        if let Some(constraints) = &deployment_details.constraints {
            info!("      <green><b>Constraints</>:");
            info!("         <green><b>Minimum</>: {}", constraints.minimum);
            info!("         <green><b>Maximum</>: {}", constraints.maximum);
            info!("         <green><b>Priority</>: {}", constraints.priority);
        } else {
            warn!("      <yellow><b>Constraints</>: None");
        }

        if let Some(scaling) = &deployment_details.scaling {
            info!("      <green><b>Scaling</>:");
            info!(
                "         <green><b>Start Threshold</>: {}%",
                scaling.start_threshold * 100.0
            );
            info!(
                "         <green><b>Stop Empty Units</>: {}",
                scaling.stop_empty_units
            );
        } else {
            warn!("      <yellow><b>Scaling</>: None");
        }

        if let Some(resources) = &deployment_details.resources {
            info!("      <green><b>Resources per Unit</>:");
            info!("         <green><b>Memory</>: {} MiB", resources.memory);
            info!("         <green><b>Swap</>: {} MiB", resources.swap);
            info!("         <green><b>CPU Cores</>: {}", resources.cpu / 100);
            info!("         <green><b>IO</>: {}", resources.io);
            info!("         <green><b>Disk Space</>: {} MiB", resources.disk);
            info!(
                "         <green><b>Addresses/Ports</>: {}",
                resources.addresses
            );
        } else {
            warn!("      <yellow><b>Resources per Unit</>: None");
        }

        if let Some(spec) = &deployment_details.spec {
            info!("      <green><b>Specification</>:");
            info!("         <green><b>Image</>: {}", spec.image);
            info!(
                "         <green><b>Max Players per Unit</>: {}",
                spec.max_players
            );
            info!("         <green><b>Settings</>:");
            for setting in &spec.settings {
                info!("            - <green>{}</>: {}", setting.key, setting.value);
            }
            info!("         <green><b>Environment Variables</>:");
            for env in &spec.environment {
                info!("            - <green>{}</>: {}", env.key, env.value);
            }
            info!(
                "         <green><b>Disk Retention</>: {}",
                spec.disk_retention.unwrap_or(0)
            );

            if let Some(fallback) = &spec.fallback {
                info!("         <green><b>Fallback</>:");
                info!("            <green><b>Enabled</>: {}", fallback.enabled);
                info!("            <green><b>Priority</>: {}", fallback.priority);
            }
        } else {
            warn!("      <yellow><b>Specification</>: None");
        }
    }
}
