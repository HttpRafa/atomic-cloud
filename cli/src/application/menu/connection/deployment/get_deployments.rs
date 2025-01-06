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
            "Requesting deployment list from controller \"{}\"...",
            profile.name
        ));

        match connection.client.get_deployments().await {
            Ok(deployments) => {
                progress.success("Deployment data retrieved successfully 👍");
                progress.end();
                Self::display_deployments(&deployments);
                MenuResult::Success
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    fn display_deployments(deployments: &[String]) {
        info!("   <blue>🖥  <b>Available Deployments</>");
        if deployments.is_empty() {
            info!("      <green><b>No deployments found.</>");
        } else {
            for deployment in deployments {
                info!("    - <green>{}</>", deployment);
            }
        }
    }
}
