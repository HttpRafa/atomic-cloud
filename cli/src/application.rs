use log::info;
use profile::Profiles;

use crate::menu::{start::StartMenu, Menu};

pub mod profile;

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
        StartMenu::show(&mut self.profiles);
        info!("Goodbye!");
    }
}
