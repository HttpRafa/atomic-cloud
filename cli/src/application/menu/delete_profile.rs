use inquire::InquireError;
use loading::Loading;
use simplelog::debug;

use crate::application::profile::Profiles;

use super::{MenuResult, MenuUtils};

pub struct DeleteProfileMenu;

impl DeleteProfileMenu {
    pub async fn show(profiles: &mut Profiles) -> MenuResult {
        let options = profiles.profiles.clone();
        match MenuUtils::select_no_help("What profile/controller do you want to delete?", options) {
            Ok(profile) => match MenuUtils::confirm("Do you really want to delete this profile?") {
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
            },
            Err(error) => match error {
                InquireError::OperationCanceled | InquireError::OperationInterrupted => MenuResult::Aborted,
                _ => MenuResult::Error(error.into())
            }
        }
    }
}
