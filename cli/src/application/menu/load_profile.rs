use inquire::Select;
use simplelog::debug;

use crate::application::profile::Profiles;

use super::{connection::ConnectionMenu, MenuResult};

pub struct LoadProfileMenu;

impl LoadProfileMenu {
    pub async fn show(profiles: &mut Profiles) -> MenuResult {
        let options = profiles.profiles.clone();
        match Select::new(
            "What profile/controller to you want to connect to?",
            options,
        )
        .prompt()
        {
            Ok(profile) => ConnectionMenu::show(profile, profiles).await,
            Err(error) => {
                debug!("{}", error);
                MenuResult::Aborted
            }
        }
    }
}
