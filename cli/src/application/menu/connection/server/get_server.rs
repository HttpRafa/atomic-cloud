use inquire::InquireError;
use loading::Loading;
use simplelog::{info, warn};

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{proto::manage::server, EstablishedConnection},
    profile::{Profile, Profiles},
};

pub struct GetServerMenu;

impl GetServerMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving available servers from controller \"{}\"",
            profile.name
        ));

        match connection.client.get_servers().await {
            Ok(servers) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();
                match MenuUtils::select_no_help("Select a server to view more details:", servers) {
                    Ok(server) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Fetching details for server \"{}\" from controller \"{}\"...",
                            profile.name, server
                        ));

                        match connection.client.get_server(&server.id).await {
                            Ok(unit) => {
                                progress.success("Details retrieved successfully 👍");
                                progress.end();
                                Self::display_details(&unit);
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

    fn display_details(server: &server::Detail) {
        info!("   <blue>🖥  <b>Server Info</>");
        info!("      <green><b>Name</>: {}", server.name);
        info!("      <green><b>UUID</>: {}", server.id);
        if let Some(group) = &server.group {
            info!("      <green><b>Group</>: {}", group);
        } else {
            info!("      <green><b>Group</>: None");
        }
        info!("      <green><b>Node</>: {}", server.name);
        if let Some(allocation) = &server.allocation {
            info!("      <green><b>Allocation</>: ");
            info!("         <green><b>Allocations</>: ");
            for port in &allocation.ports {
                info!("            - <green><b>{}:{}</>", port.host, port.port);
            }
            if let Some(resources) = allocation.resources {
                info!("         <green><b>Resources per unit</>: ");
                info!("            <green><b>Memory</>: {} MiB", resources.memory);
                info!("            <green><b>Swap</>: {} MiB", resources.swap);
                info!(
                    "            <green><b>CPU-Cores</>: {}",
                    resources.cpu / 100
                );
                info!("            <green><b>IO</>: {}", resources.io);
                info!(
                    "            <green><b>Disk space</>: {} MiB",
                    resources.disk
                );
                info!(
                    "            <green><b>Addresses/Ports</>: {}",
                    resources.ports
                );
            } else {
                warn!("         <yellow><b>Resources per unit</>: None");
            }
            if let Some(spec) = &allocation.spec {
                info!("         <green><b>Specification</>: ");
                info!("            <green><b>Image</>: {}", spec.img);
                info!("            <green><b>Settings</>: ");
                for setting in &spec.settings {
                    info!(
                        "               - <green><b>{}</>: {}",
                        setting.key, setting.value
                    );
                }
                info!("            <green><b>Environment Variables</>: ");
                for environment in &spec.env {
                    info!(
                        "               - <green><b>{}</>: {}",
                        environment.key, environment.value
                    );
                }
                info!(
                    "         <green><b>Disk Retention</>: {}",
                    spec.retention.unwrap_or(0)
                );
                if let Some(fallback) = spec.fallback {
                    info!("            <green><b>Fallback</>: ");
                    info!(
                        "               <green><b>Is fallback</>: {}",
                        fallback.enabled
                    );
                    info!("               <green><b>Priority</>: {}", fallback.prio);
                }
            } else {
                warn!("         <yellow><b>Specification</>: None");
            }
        } else {
            warn!("      <yellow><b>Scaling</>: None");
        }
        info!("      <green><b>Connected Users</>: {}", server.users);
        info!("      <green><b>Auth Token</>: {}", server.token);
        info!("      <green><b>State</>: {}", server.state);
        info!("      <green><b>Ready</>: {}", server.ready);
    }
}
