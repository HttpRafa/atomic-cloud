use inquire::{
    validator::{Validation, ValueRequiredValidator},
    Password, Text,
};
use loading::Loading;

use crate::{
    application::profile::{Profile, Profiles},
    VERSION,
};

use super::{MenuResult, MenuUtils};

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
            Err(error) => return MenuUtils::handle_error(error),
        };

        let authorization = match Password::new("What is the authorization token for this profile?")
            .with_help_message(
                "Example: actl_9f6e44488bb64726a70dd90df2a387484029299ad3a94f97bce1df0d3a6535d2",
            )
            .with_validator(ValueRequiredValidator::default())
            .prompt()
        {
            Ok(authorization) => authorization,
            Err(error) => return MenuUtils::handle_error(error),
        };

        let url = match MenuUtils::parsed_value(
            "What is the URL for this profile?",
            "Example: https://cloud.your-network.net",
            "Please enter a valid URL",
        ) {
            Ok(url) => url,
            Err(error) => return MenuUtils::handle_error(error),
        };

        let progress = Loading::default();
        let profile = Profile::new(&name, &authorization, url);
        progress.text(format!(
            "Connecting to the controller \"{}\" at {} to verify the profile",
            profile.name, profile.url
        ));
        match profile.establish_connection().await {
            Ok(connection) => {
                if connection.incompatible {
                    progress.warn(format!("The controller is running an incompatible protocol version {} compared to this client's version {}", connection.protocol, VERSION.protocol));
                }
            }
            Err(error) => {
                progress.fail(format!("Failed to connect to the controller: {}", error));
                progress.end();
                return MenuResult::Failed(error);
            }
        }
        progress.text(format!("Saving profile \"{}\"", name));
        if let Err(error) = profiles.create_profile(&profile).await {
            progress.fail(format!("Failed to create profile: {}", error));
            progress.end();
            return MenuResult::Failed(error);
        }
        progress.success("Profile created successfully");
        progress.end();
        MenuResult::Success
    }
}
