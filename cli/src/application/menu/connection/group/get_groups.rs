use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

pub struct GetGroupsMenu;

impl GetGroupsMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Requesting cloudGroup list from controller \"{}\"...",
            profile.name
        ));

        match connection.client.get_groups().await {
            Ok(groups) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();
                Self::display_groups(&groups);
                MenuResult::Success
            }
            Err(error) => {
                progress.fail(format!("{error}"));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    fn display_groups(groups: &[String]) {
        info!("   <blue>🖥  <b>Available Groups</>");
        if groups.is_empty() {
            info!("      <green><b>No groups found.</>");
        } else {
            for cloudGroup in groups {
                info!("    - <green>{}</>", cloudGroup);
            }
        }
    }
}
