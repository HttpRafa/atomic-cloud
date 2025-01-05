use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

pub struct GetUnitsMenu;

impl GetUnitsMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Sending request to controller \"{}\"",
            profile.name
        ));

        match connection.client.get_units().await {
            Ok(units) => {
                progress.success("Data received 👍");
                progress.end();
                info!("   <blue>🖥  <b>Units</>");
                if units.is_empty() {
                    info!("      <green><b>No units found</>");
                } else {
                    for unit in units {
                        info!(
                            "    - <green>{}</>@<cyan>{}</> (<blue>{}</>)",
                            unit.name, unit.cloudlet, unit.uuid
                        );
                    }
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
}
