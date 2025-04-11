use anyhow::Result;
use common::error::FancyError;
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
    pub async fn new() -> Result<Self> {
        Ok(Self {
            profiles: Profiles::init().await?,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            match StartMenu::show(&mut self.profiles).await {
                MenuResult::Exit => break,
                MenuResult::Failed(error) => FancyError::print_fancy(&error, false),
                _ => {}
            }
        }
        info!("<blue>ℹ</> Goodbye!");

        Ok(())
    }
}
