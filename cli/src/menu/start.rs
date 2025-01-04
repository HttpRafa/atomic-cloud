use std::{
    fmt::{Display, Formatter},
    vec,
};

use inquire::Select;
use log::error;

use crate::application::profile::Profiles;

use super::{
    create_profile::CreateProfileMenu, delete_profile::DeleteProfileMenu,
    load_profile::LoadProfileMenu, Menu, MenuResult,
};

enum Selection {
    LoadProfile,
    CreateProfile,
    DeleteProfile,
    Exit,
}

impl Display for Selection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::LoadProfile => write!(f, "Connect to existing controller"),
            Selection::CreateProfile => write!(f, "Add new controller"),
            Selection::DeleteProfile => write!(f, "Remove existing controller"),
            Selection::Exit => write!(f, "Close application"),
        }
    }
}

pub struct StartMenu;

impl Menu for StartMenu {
    fn show(profiles: &mut Profiles) -> MenuResult {
        match Select::new(
            "What do you want to do?",
            vec![
                Selection::LoadProfile,
                Selection::CreateProfile,
                Selection::DeleteProfile,
                Selection::Exit,
            ],
        )
        .prompt()
        {
            Ok(selection) => match selection {
                Selection::LoadProfile => LoadProfileMenu::show(profiles),
                Selection::CreateProfile => CreateProfileMenu::show(profiles),
                Selection::DeleteProfile => DeleteProfileMenu::show(profiles),
                Selection::Exit => MenuResult::Success,
            },
            Err(error) => {
                error!(
                    "Ops. Something went wrong while evaluating your input | {}",
                    error
                );
                MenuResult::Failed
            }
        }
    }
}
