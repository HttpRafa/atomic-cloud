use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{proto::manage::node, EstablishedConnection},
    profile::{Profile, Profiles},
};

pub struct GetNodeMenu;

impl GetNodeMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving available nodes from controller \"{}\"...",
            profile.name
        ));

        match connection.client.get_nodes().await {
            Ok(nodes) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();
                match MenuUtils::select_no_help("Select a nodes to view more details:", nodes) {
                    Ok(node) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Fetching details for node \"{}\" from controller \"{}\"...",
                            node, profile.name
                        ));

                        match connection.client.get_node(&node).await {
                            Ok(details) => {
                                progress.success("Node details retrieved successfully 👍");
                                progress.end();
                                Self::display_details(&details);
                                MenuResult::Success
                            }
                            Err(error) => {
                                progress.fail(format!("{}", error));
                                progress.end();
                                MenuResult::Failed(error)
                            }
                        }
                    }
                    Err(error) => MenuUtils::handle_error(error),
                }
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    fn display_details(node: &node::Item) {
        info!("   <blue>🖥  <b>Node Information</>");
        info!("      <green><b>Name</>: {}", node.name);
        info!("      <green><b>Plugin</>: {}", node.plugin);
        if let Some(memory) = &node.memory {
            info!("      <green><b>Memory</>: {} MiB", memory);
        }
        if let Some(max) = &node.max {
            info!("      <green><b>Max Servers</>: {} Units", max);
        }
        if let Some(child) = &node.child {
            info!("      <green><b>Child Node</>: {}", child);
        }
        info!("      <green><b>Controller Address</>: {}", node.ctrl_addr);
    }
}
