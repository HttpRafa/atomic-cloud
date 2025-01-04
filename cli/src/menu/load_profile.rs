use std::{thread, time::Duration};

use inquire::Select;
use loading::Loading;
use log::debug;

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
            Ok(profile) => {
                let progress = Loading::default();
                progress.text(format!(
                    "Connecting to controller \"{}\" at {}",
                    profile.name, profile.url
                ));
                thread::sleep(Duration::from_secs(3));
                progress.fail("Not implemented yet");
                progress.end();
                MenuResult::Success
            }
            Err(error) => {
                debug!("{}", error);
                MenuResult::Aborted
            }
        }
    }
}
