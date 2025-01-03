use profile::Profiles;
use prompt::Prompt;

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
        Prompt::select_profile(&self.profiles);
    }
}
