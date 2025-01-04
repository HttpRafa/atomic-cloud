use inquire::{
    validator::{Validation, ValueRequiredValidator},
    Password, Text,
};
use log::error;
use url::Url;

use crate::application::profile::{Profile, Profiles};

use super::{Menu, MenuResult};

pub struct CreateProfileMenu;

impl Menu for CreateProfileMenu {
    fn show(profiles: &mut Profiles) -> MenuResult {
        let name = match Text::new("How do you want to name this profile?")
            .with_validator(ValueRequiredValidator::default())
            .prompt()
        {
            Ok(name) => name,
            Err(error) => {
                error!(
                    "Ops. Something went wrong while evaluating your input | {}",
                    error
                );
                return MenuResult::Failed;
            }
        };

        let authorization = match Password::new("What is the authorization token for this profile?")
            .with_validator(ValueRequiredValidator::default())
            .prompt()
        {
            Ok(authorization) => authorization,
            Err(error) => {
                error!(
                    "Ops. Something went wrong while evaluating your input | {}",
                    error
                );
                return MenuResult::Failed;
            }
        };

        let url = match Text::new("What is the URL for this profile?")
            .with_validator(|url: &str| match Url::parse(url) {
                Ok(_) => Ok(Validation::Valid),
                Err(error) => Ok(Validation::Invalid(error.into())),
            })
            .prompt()
        {
            Ok(url) => Url::parse(&url).expect("Ops. That was unexpected"),
            Err(error) => {
                error!(
                    "Ops. Something went wrong while evaluating your input | {}",
                    error
                );
                return MenuResult::Failed;
            }
        };

        let profile = Profile::new(&name, &authorization, url);
        if let Err(error) = profiles.create_profile(&profile) {
            error!("Failed to create profile: {}", error);
            return MenuResult::Failed;
        }
        MenuResult::Success
    }
}
