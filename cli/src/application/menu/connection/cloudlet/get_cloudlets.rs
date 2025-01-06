use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

pub struct GetCloudletsMenu;

impl GetCloudletsMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Requesting cloudlet list from controller \"{}\"...",
            profile.name
        ));

        match connection.client.get_cloudlets().await {
            Ok(cloudlets) => {
                progress.success("Cloudlet data retrieved successfully 👍");
                progress.end();
                Self::display_cloudlets(&cloudlets);
                MenuResult::Success
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    fn display_cloudlets(cloudlets: &[String]) {
        info!("   <blue>🖥  <b>Available Cloudlets</>");
        if cloudlets.is_empty() {
            info!("      <green><b>No cloudlets found.</>");
        } else {
            for cloudlet in cloudlets {
                info!("    - <green>{}</>", cloudlet);
            }
        }
    }
}
