use loading::Loading;
use simplelog::info;

use crate::application::{
    menu::MenuResult,
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

pub struct GetDeploymentsMenu;

impl GetDeploymentsMenu {
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

        match connection.client.get_deployments().await {
            Ok(deployments) => {
                progress.success("Data received 👍");
                progress.end();
                info!("   <blue>🖥  <b>Deployments</>");
                if deployments.is_empty() {
                    info!("      <green><b>No deployments found</>");
                } else {
                    for deployment in deployments {
                        info!("    - <green>{}</>", deployment);
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
