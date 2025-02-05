use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

pub struct GetNodesMenu;

impl GetNodesMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Requesting nodes list from controller \"{}\"...",
            profile.name
        ));

        match connection.client.get_nodes().await {
            Ok(nodes) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();
                Self::display_details(&nodes);
                MenuResult::Success
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    fn display_details(nodes: &[String]) {
        info!("   <blue>🖥  <b>Available Nodes</>");
        if nodes.is_empty() {
            info!("      <green><b>No nodes found.</>");
        } else {
            for node in nodes {
                info!("    - <green>{}</>", node);
            }
        }
    }
}
