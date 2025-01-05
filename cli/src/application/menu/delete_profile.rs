use inquire::{Confirm, Select};
use loading::Loading;
use simplelog::debug;

use crate::application::profile::Profiles;

use super::MenuResult;

pub struct DeleteProfileMenu;

impl DeleteProfileMenu {
    pub async fn show(profiles: &mut Profiles) -> MenuResult {
        let options = profiles.profiles.clone();
        match Select::new("What profile/controller do you want to delete?", options).prompt() {
            Ok(profile) => {
                match Confirm::new("Do you really want to delete this profile?")
                    .with_help_message("Type y or n")
                    .prompt()
                {
                    Ok(true) => {
                        let progress = Loading::default();
                        progress.text(format!("Deleting profile \"{}\"", profile.name));
                        match profiles.delete_profile(&profile) {
                            Ok(_) => {
                                progress.success("Profile deleted successfully");
                                progress.end();
                                MenuResult::Success
                            }
                            Err(err) => {
                                progress.fail(format!(
                                    "Ops. Something went wrong while deleting the profile | {}",
                                    err
                                ));
                                progress.end();
                                MenuResult::Failed
                            }
                        }
                    }
                    Ok(false) | Err(_) => MenuResult::Aborted,
                }
            }
            Err(err) => {
                debug!("{}", err);
                MenuResult::Aborted
            }
        }
    }
}
