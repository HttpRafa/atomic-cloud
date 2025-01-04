use log::info;
use profile::Profiles;

use crate::menu::{start::StartMenu, Menu, MenuResult};

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
        loop {
            if StartMenu::show(&mut self.profiles) == MenuResult::Exit {
                break;
            }
        }
        info!("ℹ Goodbye!");
    }
}
