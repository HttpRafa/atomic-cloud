use std::{
    fmt::{Display, Formatter},
    vec,
};

use inquire::Select;
use log::debug;

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
            Selection::LoadProfile => write!(f, "🖧 | Connect to existing controller"),
            Selection::CreateProfile => write!(f, "+ | Add new controller"),
            Selection::DeleteProfile => write!(f, "🗑 | Remove existing controller"),
            Selection::Exit => write!(f, "✖ | Close application"),
        }
    }
}

pub struct StartMenu;

impl Menu for StartMenu {
    fn show(profiles: &mut Profiles) -> MenuResult {
        let mut options = vec![];

        {
            let amount = profiles.profiles.len();
            if amount > 0 {
                options.push(Selection::LoadProfile);
                options.push(Selection::DeleteProfile);
            }
            options.push(Selection::CreateProfile);
            options.push(Selection::Exit);
        }

        match Select::new("What do you want to do?", options).prompt() {
            Ok(selection) => match selection {
                Selection::LoadProfile => LoadProfileMenu::show(profiles),
                Selection::CreateProfile => CreateProfileMenu::show(profiles),
                Selection::DeleteProfile => DeleteProfileMenu::show(profiles),
                Selection::Exit => MenuResult::Exit,
            },
            Err(error) => {
                debug!("{}", error);
                MenuResult::Exit
            }
        }
    }
}
