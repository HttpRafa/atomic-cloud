use inquire::InquireError;
use loading::Loading;
use simplelog::{info, warn};

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{proto::manage::group, EstablishedConnection},
    profile::{Profile, Profiles},
};

pub struct GetGroupMenu;

impl GetGroupMenu {
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

        match connection.client.get_groups().await {
            Ok(deployments) => {
                progress.success("Data retrieved successfully 👍");
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

                        match connection.client.get_group(&deployment).await {
                            Ok(deployment_details) => {
                                progress.success("Group details retrieved successfully 👍");
                                progress.end();

                                Self::display_details(&deployment_details);
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

    fn display_details(group: &group::Item) {
        info!("   <blue>🖥  <b>Group Details</>");
        info!("      <green><b>Name</>: {}", group.name);

        if !group.nodes.is_empty() {
            info!("      <green><b>Nodes</>:");
            for node in &group.nodes {
                info!("         - <green>{}</>", node);
            }
        } else {
            warn!("      <yellow><b>Nodes</>: None");
        }

        if let Some(constraints) = &group.constraints {
            info!("      <green><b>Constraints</>:");
            info!("         <green><b>Minimum</>: {}", constraints.min);
            info!("         <green><b>Maximum</>: {}", constraints.max);
            info!("         <green><b>Priority</>: {}", constraints.prio);
        } else {
            warn!("      <yellow><b>Constraints</>: None");
        }

        if let Some(scaling) = &group.scaling {
            info!("      <green><b>Scaling</>:");
            info!(
                "         <green><b>Start Threshold</>: {}%",
                scaling.start_threshold * 100.0
            );
            info!("         <green><b>Stop Empty</>: {}", scaling.stop_empty);
        } else {
            warn!("      <yellow><b>Scaling</>: None");
        }

        if let Some(resources) = &group.resources {
            info!("      <green><b>Resources per Unit</>:");
            info!("         <green><b>Memory</>: {} MiB", resources.memory);
            info!("         <green><b>Swap</>: {} MiB", resources.swap);
            info!("         <green><b>CPU Cores</>: {}", resources.cpu / 100);
            info!("         <green><b>IO</>: {}", resources.io);
            info!("         <green><b>Disk Space</>: {} MiB", resources.disk);
            info!("         <green><b>Addresses/Ports</>: {}", resources.ports);
        } else {
            warn!("      <yellow><b>Resources per Unit</>: None");
        }

        if let Some(spec) = &group.spec {
            info!("      <green><b>Specification</>:");
            info!("         <green><b>Image</>: {}", spec.img);
            info!(
                "         <green><b>Max Players per Unit</>: {}",
                spec.max_players
            );
            info!("         <green><b>Settings</>:");
            for setting in &spec.settings {
                info!("            - <green>{}</>: {}", setting.key, setting.value);
            }
            info!("         <green><b>Environment Variables</>:");
            for env in &spec.env {
                info!("            - <green>{}</>: {}", env.key, env.value);
            }
            info!(
                "         <green><b>Disk Retention</>: {}",
                spec.retention.unwrap_or(0)
            );

            if let Some(fallback) = &spec.fallback {
                info!("         <green><b>Fallback</>:");
                info!("            <green><b>Enabled</>: {}", fallback.enabled);
                info!("            <green><b>Priority</>: {}", fallback.prio);
            }
        } else {
            warn!("      <yellow><b>Specification</>: None");
        }
    }
}
