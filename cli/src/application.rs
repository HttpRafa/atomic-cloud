use log::error;
use profile::{Profile, Profiles};
use prompt::{Prompt, Selection};

mod profile;
mod prompt;

pub struct Cli {
    profiles: Profiles,
}

impl Cli {
    pub async fn new() -> Cli {
        Cli {
            profiles: Profiles::load_all(),
        }
    }

    pub async fn start(&mut self) {
        match Prompt::select_profile(&self.profiles) {
            Selection::Some(profile) => {
                self.start_profile_menu(profile).await;
            }
            Selection::Create => {
                if let Some(profile) = Prompt::collect_profile_information() {
                    if let Err(error) = self.profiles.create_profile(&profile) {
                        error!("Failed to create profile: {}", error);
                    } else {
                        self.start_profile_menu(&profile).await;
                    }
                }
            }
            Selection::Delete => {}
            Selection::None => {}
        }
    }

    async fn start_profile_menu(&self, _profile: &Profile) {
        // TODO: Implement profile menu
    }
}
