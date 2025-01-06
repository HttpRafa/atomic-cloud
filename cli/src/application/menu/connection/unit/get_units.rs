use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::{proto::unit_management::SimpleUnitValue, EstablishedConnection},
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
            "Requesting unit list from controller \"{}\"",
            profile.name
        ));

        match connection.client.get_units().await {
            Ok(units) => {
                progress.success("Unit data retrieved successfully 👍");
                progress.end();
                Self::display_details(&units);
                MenuResult::Success
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    fn display_details(units: &[SimpleUnitValue]) {
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
    }
}
