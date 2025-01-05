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
            "Sending request to controller \"{}\"",
            profile.name
        ));

        match connection.client.get_cloudlets().await {
            Ok(cloudlets) => {
                progress.success("Data received 👍");
                progress.end();
                info!("   <blue>🖥  <b>Cloudlets</>");
                if cloudlets.is_empty() {
                    info!("      <green><b>No cloudlets found</>");
                } else {
                    for cloudlet in cloudlets {
                        info!("    - <green>{}</>", cloudlet);
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
