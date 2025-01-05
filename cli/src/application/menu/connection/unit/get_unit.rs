use std::fmt::Display;

use inquire::Select;
use loading::Loading;
use simplelog::{info, warn};

use crate::application::{
    menu::MenuResult,
    network::{proto::unit_management::SimpleUnitValue, EstablishedConnection},
    profile::{Profile, Profiles},
};

impl Display for SimpleUnitValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct GetUnitMenu;

impl GetUnitMenu {
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
        
        match connection.client.get_units().await {
            Ok(units) => {
                progress.success("Data received 👍");
                progress.end();
                match Select::new("From what unit do want more information?", units)
                    .prompt()
                {
                    Ok(unit) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Getting information from controller \"{}\" about unit \"{}\"",
                            profile.name,
                            unit
                        ));
                
                        match connection.client.get_unit(&unit.uuid).await {
                            Ok(unit) => {
                                progress.success("Data received 👍");
                                progress.end();
                                info!("   <blue>🖥  <b>Unit Info</>");
                                info!("      <green><b>Name</>: {}", unit.name);
                                info!("      <green><b>UUID</>: {}", unit.uuid);
                                if let Some(deployment) = unit.deployment {
                                    info!("      <green><b>Deployment</>: {}", deployment);
                                } else {
                                    info!("      <green><b>Deployment</>: None");
                                }
                                info!("      <green><b>Cloudlet</>: {}", unit.cloudlet);
                                if let Some(allocation) = unit.allocation {
                                    info!("      <green><b>Allocation</>: ");
                                    info!("         <green><b>Allocations</>: ");
                                    for address in allocation.addresses {
                                        info!("            - <green><b>{}:{}</>", address.ip, address.port);
                                    }
                                    if let Some(resources) = allocation.resources {
                                        info!("         <green><b>Resources per unit</>: ");
                                        info!("            <green><b>Memory</>: {} MiB", resources.memory);
                                        info!("            <green><b>Swap</>: {} MiB", resources.swap);
                                        info!("            <green><b>CPU-Cores</>: {}", resources.cpu / 100);
                                        info!("            <green><b>IO</>: {}", resources.io);
                                        info!("            <green><b>Disk space</>: {} MiB", resources.disk);
                                        info!("            <green><b>Addresses/Ports</>: {}", resources.addresses);
                                    } else {
                                        warn!("         <yellow><b>Resources per unit</>: None");
                                    }
                                    if let Some(spec) = allocation.spec {
                                        info!("         <green><b>Specification</>: ");
                                        info!("            <green><b>Image</>: {}", spec.image);
                                        info!("            <green><b>Settings</>: ");
                                        for setting in spec.settings {
                                            info!("               - <green><b>{}</>: {}", setting.key, setting.value);
                                        }
                                        info!("            <green><b>Environment Variables</>: ");
                                        for environment in spec.environment {
                                            info!("               - <green><b>{}</>: {}", environment.key, environment.value);
                                        }
                                        info!("         <green><b>Disk Retention</>: {}", spec.disk_retention.unwrap_or(0));
                                        if let Some(fallback) = spec.fallback {
                                            info!("            <green><b>Fallback</>: ");
                                            info!("               <green><b>Is fallback</>: {}", fallback.enabled);
                                            info!("               <green><b>Priority</>: {}", fallback.priority);
                                        }
                                    } else {
                                        warn!("         <yellow><b>Specification</>: None");
                                    }
                                } else {
                                    warn!("      <yellow><b>Scaling</>: None");
                                }
                                info!("      <green><b>Connected Users</>: {}", unit.connected_users);
                                info!("      <green><b>Auth Token</>: {}", unit.auth_token);
                                info!("      <green><b>State</>: {}", unit.state);
                                info!("      <green><b>Rediness</>: {}", unit.rediness);
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
                    Err(_) => MenuResult::Aborted
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
