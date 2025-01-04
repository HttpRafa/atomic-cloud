use inquire::Select;
use log::error;

use crate::application::profile::Profiles;

use super::{Menu, MenuResult};

pub struct LoadProfileMenu;

impl Menu for LoadProfileMenu {
    fn show(profiles: &mut Profiles) -> MenuResult {
        let options = profiles.profiles.clone();
        match Select::new(
            "What profile/controller to you want to connect to?",
            options,
        )
        .prompt()
        {
            Ok(_selection) => MenuResult::Success,
            Err(error) => {
                error!(
                    "Ops. Something went wrong while evaluating your input | {}",
                    error
                );
                MenuResult::Failed
            }
        }
    }
}
