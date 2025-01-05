use inquire::{
    validator::{Validation, ValueRequiredValidator},
    Password, Text,
};
use loading::Loading;
use log::debug;
use url::Url;

use crate::{
    application::profile::{Profile, Profiles},
    VERSION,
};

use super::MenuResult;

pub struct CreateProfileMenu;

impl CreateProfileMenu {
    pub async fn show(profiles: &mut Profiles) -> MenuResult {
        let mut prompt = Text::new("How do you want to name this profile?")
            .with_help_message("Example: Production Controller")
            .with_validator(ValueRequiredValidator::default());
        {
            let used_profiles = profiles.profiles.clone();
            prompt = prompt.with_validator(move |name: &str| {
                if Profiles::already_exists(&used_profiles, name) {
                    Ok(Validation::Invalid(
                        "Profile with this name already exists".into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            });
        }
        let name = match prompt.prompt() {
            Ok(name) => name,
            Err(error) => {
                debug!("{}", error);
                return MenuResult::Aborted;
            }
        };

        let authorization = match Password::new("What is the authorization token for this profile?")
            .with_help_message(
                "Example: actl_9f6e44488bb64726a70dd90df2a387484029299ad3a94f97bce1df0d3a6535d2",
            )
            .with_validator(ValueRequiredValidator::default())
            .prompt()
        {
            Ok(authorization) => authorization,
            Err(error) => {
                debug!("{}", error);
                return MenuResult::Aborted;
            }
        };

        let url = match Text::new("What is the URL for this profile?")
            .with_help_message("Example: https://cloud.your-network.net")
            .with_validator(|url: &str| match Url::parse(url) {
                Ok(_) => Ok(Validation::Valid),
                Err(error) => Ok(Validation::Invalid(error.into())),
            })
            .prompt()
        {
            Ok(url) => Url::parse(&url).expect("Ops. That was unexpected"),
            Err(error) => {
                debug!("{}", error);
                return MenuResult::Aborted;
            }
        };

        let progress = Loading::default();
        let profile = Profile::new(&name, &authorization, url);
        progress.text(format!(
            "Connecting to the controller \"{}\" at {} to verify the profile",
            profile.name, profile.url
        ));
        match profile.establish_connection().await {
            Ok(connection) => {
                if connection.outdated {
                    progress.warn(format!("The controller is running an outdated protocol version {} compared to this client running {}", connection.protocol, VERSION.protocol));
                }
            }
            Err(error) => {
                progress.fail(format!("Failed to connect to the controller: {}", error));
                progress.end();
                return MenuResult::Failed;
            }
        }
        progress.text(format!("Saving profile \"{}\"", name));
        if let Err(error) = profiles.create_profile(&profile) {
            progress.fail(format!("Failed to create profile: {}", error));
            progress.end();
            return MenuResult::Failed;
        }
        progress.success("Profile created successfully");
        progress.end();
        MenuResult::Success
    }
}
