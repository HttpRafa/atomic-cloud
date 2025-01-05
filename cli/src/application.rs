use menu::{start::StartMenu, MenuResult};
use profile::Profiles;
use simplelog::info;

mod menu;
mod network;
mod profile;

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
        loop {
            if StartMenu::show(&mut self.profiles).await == MenuResult::Exit {
                break;
            }
        }
        info!("<blue>ℹ</> Goodbye!");
    }
}
