use anyhow::Result;
use common::error::{FancyError};
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

    pub async fn start(&mut self) -> Result<()> {
        loop {
            match StartMenu::show(&mut self.profiles).await {
                MenuResult::Exit => break,
                MenuResult::Error(error) => { FancyError::print_fancy(&error, false); break; },
                _ => {}
            }
        }
        info!("<blue>ℹ</> Goodbye!");

        Ok(())
    }
}
