use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::{proto::manage::server, EstablishedConnection},
    profile::{Profile, Profiles},
};

pub struct GetServersMenu;

impl GetServersMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Requesting server list from controller \"{}\"",
            profile.name
        ));

        match connection.client.get_servers().await {
            Ok(servers) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();
                Self::display_details(&servers);
                MenuResult::Success
            }
            Err(error) => {
                progress.fail(format!("{error}"));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    fn display_details(servers: &[server::Short]) {
        info!("   <blue>🖥  <b>Servers</>");
        if servers.is_empty() {
            info!("      <green><b>No server found</>");
        } else {
            for server in servers {
                info!(
                    "    - <green>{}</>@<cyan>{}</> (<blue>{}</>)",
                    server.name, server.node, server.id
                );
            }
        }
    }
}
