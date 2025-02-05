use inquire::InquireError;

use crate::application::profile::Profiles;

use super::{connection::ConnectionMenu, MenuResult, MenuUtils};

pub struct LoadProfileMenu;

impl LoadProfileMenu {
    pub async fn show(profiles: &mut Profiles) -> MenuResult {
        let options = profiles.profiles.clone();
        match MenuUtils::select_no_help(
            "What profile/controller to you want to connect to?",
            options,
        ) {
            Ok(profile) => ConnectionMenu::show(profile, profiles).await,
            Err(error) => match error {
                InquireError::OperationCanceled | InquireError::OperationInterrupted => {
                    MenuResult::Aborted
                }
                _ => MenuResult::Failed(error.into()),
            },
        }
    }
}
