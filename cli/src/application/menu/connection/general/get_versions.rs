use anyhow::Result;
use loading::Loading;
use simplelog::info;

use crate::{
    application::{
        menu::MenuResult,
        network::EstablishedConnection,
        profile::{Profile, Profiles},
    },
    VERSION,
};

pub struct GetVersionsMenu;

impl GetVersionsMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Requesting version info from controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok((version, protocol)) => {
                progress.success("Version data retrieved successfully 👍");
                progress.end();
                info!("   <blue>🖥  <b>Controller Info</>");
                info!("      <green><b>Version</>: {}", version);
                info!("      <green><b>Protocol version</>: {}", protocol);
                info!("   <blue>🖳  <b>Client Info</>");
                info!("      <green><b>Version</>: {}", VERSION);
                info!("      <green><b>Protocol version</>: {}", VERSION.protocol);
                MenuResult::Success
            }
            Err(err) => {
                progress.fail(format!(
                    "Something went wrong while retrieving version info from the controller: {}",
                    err
                ));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<(String, u32)> {
        let version = connection.client.get_controller_version().await?;
        let protocol = connection.client.get_protocol_version().await?;
        Ok((version, protocol))
    }
}
