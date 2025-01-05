use crate::application::{menu::MenuResult, network::EstablishedConnection, profile::{Profile, Profiles}};

pub struct ConnectionStartMenu;

impl ConnectionStartMenu {
    pub async fn show(_profile: Profile, _connection: EstablishedConnection, _profiles: &mut Profiles) -> MenuResult {
        MenuResult::Success
    }
}