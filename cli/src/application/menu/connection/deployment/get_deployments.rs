use crate::application::{menu::MenuResult, network::EstablishedConnection, profile::{Profile, Profiles}};

pub struct GetDeploymentsMenu;

impl GetDeploymentsMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        profiles: &mut Profiles,
    ) -> MenuResult {
        MenuResult::Success
    }
}