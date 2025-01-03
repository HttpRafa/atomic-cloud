use std::{
    fmt::{Display, Formatter},
    process::exit,
};

use inquire::Select;
use log::{error, info};

use super::profile::Profiles;

enum Selection<T> {
    Some(T),
    Create,
}

impl<T: Display> Display for Selection<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::Some(value) => write!(f, "{}", value),
            Selection::Create => write!(f, "Create new"),
        }
    }
}

pub struct Prompt;

impl Prompt {
    pub fn select_profile(profiles: &Profiles) {
        let mut options = profiles
            .profiles
            .iter()
            .map(Selection::Some)
            .collect::<Vec<_>>();
        options.push(Selection::Create);
        match Select::new(
            "What profile/controller to you want to interact with?",
            options,
        )
        .prompt()
        {
            Ok(selection) => match selection {
                Selection::Some(profile) => {
                    info!("Selected profile '{}'", profile);
                }
                Selection::Create => {
                    info!("Creating new profile");
                }
            },
            Err(error) => {
                error!("The CLI requires a profile to be selected | {}", error);
                exit(1);
            }
        }
    }
}
