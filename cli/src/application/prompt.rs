use std::fmt::{Display, Formatter};

use inquire::{
    validator::{Validation, ValueRequiredValidator},
    Password, Select, Text,
};
use log::error;
use url::Url;

use super::profile::{Profile, Profiles};

pub enum Selection<T> {
    Some(T),
    Create,
    Delete,
    None,
}

impl<T: Display> Display for Selection<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::Some(value) => write!(f, "{}", value),
            Selection::Create => write!(f, "Create new"),
            Selection::Delete => write!(f, "Delete one"),
            Selection::None => write!(f, ""),
        }
    }
}

pub struct Prompt;

impl Prompt {
    /* Profiles */
    pub fn select_profile(profiles: &Profiles) -> Selection<&Profile> {
        let mut options = profiles
            .profiles
            .iter()
            .map(Selection::Some)
            .collect::<Vec<_>>();
        options.push(Selection::Create);
        options.push(Selection::Delete);
        match Select::new(
            "What profile/controller to you want to interact with?",
            options,
        )
        .prompt()
        {
            Ok(selection) => selection,
            Err(error) => {
                error!(
                    "Ops. Something went wrong while evaluating your input | {}",
                    error
                );
                Selection::None
            }
        }
    }
    pub fn collect_profile_information() -> Option<Profile> {
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
                return None;
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
                return None;
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
                return None;
            }
        };

        Some(Profile::new(&name, &authorization, url))
    }
}
