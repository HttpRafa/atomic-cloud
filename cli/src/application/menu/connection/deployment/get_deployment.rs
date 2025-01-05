use inquire::Select;
use loading::Loading;
use simplelog::{info, warn};

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
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
            "Getting all available deployments from controller \"{}\"",
            profile.name
        ));

        match connection.client.get_deployments().await {
            Ok(deployments) => {
                progress.success("Data received 👍");
                progress.end();
                match Select::new(
                    "From what deployment do want more information?",
                    deployments,
                )
                .prompt()
                {
                    Ok(deployment) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Getting information from controller \"{}\" about deployment \"{}\"",
                            profile.name, deployment
                        ));

                        match connection.client.get_deployment(&deployment).await {
                            Ok(deployment) => {
                                progress.success("Data received 👍");
                                progress.end();
                                info!("   <blue>🖥  <b>Deployment Info</>");
                                info!("      <green><b>Name</>: {}", deployment.name);
                                if !deployment.cloudlets.is_empty() {
                                    info!("      <green><b>Cloudlets</>: ");
                                    for cloudlet in deployment.cloudlets {
                                        info!("         - <green><b>{}</>", cloudlet);
                                    }
                                } else {
                                    warn!("      <yellow><b>Cloudlets</>: None");
                                }
                                if let Some(constraints) = deployment.constraints {
                                    info!("      <green><b>Constraints</>: ");
                                    info!("         <green><b>Minimum</>: {}", constraints.minimum);
                                    info!("         <green><b>Maximum</>: {}", constraints.maximum);
                                    info!(
                                        "         <green><b>Priority</>: {}",
                                        constraints.priority
                                    );
                                } else {
                                    warn!("      <yellow><b>Constraints</>: None");
                                }
                                if let Some(scaling) = deployment.scaling {
                                    info!("      <green><b>Scaling</>: ");
                                    info!(
                                        "         <green><b>Max Players per unit</>: {}",
                                        scaling.max_players
                                    );
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
                                if let Some(resources) = deployment.resources {
                                    info!("      <green><b>Resources per unit</>: ");
                                    info!("         <green><b>Memory</>: {} MiB", resources.memory);
                                    info!("         <green><b>Swap</>: {} MiB", resources.swap);
                                    info!(
                                        "         <green><b>CPU-Cores</>: {}",
                                        resources.cpu / 100
                                    );
                                    info!("         <green><b>IO</>: {}", resources.io);
                                    info!(
                                        "         <green><b>Disk space</>: {} MiB",
                                        resources.disk
                                    );
                                    info!(
                                        "         <green><b>Addresses/Ports</>: {}",
                                        resources.addresses
                                    );
                                } else {
                                    warn!("      <yellow><b>Resources per unit</>: None");
                                }
                                if let Some(spec) = deployment.spec {
                                    info!("      <green><b>Specification</>: ");
                                    info!("         <green><b>Image</>: {}", spec.image);
                                    info!("         <green><b>Settings</>: ");
                                    for setting in spec.settings {
                                        info!(
                                            "            - <green><b>{}</>: {}",
                                            setting.key, setting.value
                                        );
                                    }
                                    info!("         <green><b>Environment Variables</>: ");
                                    for environment in spec.environment {
                                        info!(
                                            "            - <green><b>{}</>: {}",
                                            environment.key, environment.value
                                        );
                                    }
                                    info!(
                                        "         <green><b>Disk Retention</>: {}",
                                        spec.disk_retention.unwrap_or(0)
                                    );
                                    if let Some(fallback) = spec.fallback {
                                        info!("         <green><b>Fallback</>: ");
                                        info!(
                                            "            <green><b>Is fallback</>: {}",
                                            fallback.enabled
                                        );
                                        info!(
                                            "            <green><b>Priority</>: {}",
                                            fallback.priority
                                        );
                                    }
                                } else {
                                    warn!("      <yellow><b>Specification</>: None");
                                }
                                MenuResult::Success
                            }
                            Err(err) => {
                                progress.fail(format!(
                                    "Ops. Something went wrong while getting the required version information from the controller | {}",
                                    err
                                ));
                                progress.end();
                                MenuResult::Failed
                            }
                        }
                    }
                    Err(_) => MenuResult::Aborted,
                }
            }
            Err(err) => {
                progress.fail(format!(
                    "Ops. Something went wrong while getting the required version information from the controller | {}",
                    err
                ));
                progress.end();
                MenuResult::Failed
            }
        }
    }
}
