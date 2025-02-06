use loading::Loading;

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
                        Err(error) => {
                            progress.fail(format!(
                                "Ops. Something went wrong while deleting the profile | {}",
                                error
                            ));
                            progress.end();
                            MenuResult::Failed(error)
                        }
                    }
                }
                Ok(false) => MenuResult::Aborted,
                Err(error) => MenuUtils::handle_error(error),
            },
            Err(error) => MenuUtils::handle_error(error),
        }
    }
}
