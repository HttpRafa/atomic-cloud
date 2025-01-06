use std::{
    fmt::{Display, Formatter},
    vec,
};

use simplelog::debug;

use crate::application::profile::Profiles;

use super::{
    create_profile::CreateProfileMenu, delete_profile::DeleteProfileMenu,
    load_profile::LoadProfileMenu, MenuResult, MenuUtils,
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

impl StartMenu {
    pub async fn show(profiles: &mut Profiles) -> MenuResult {
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

        match MenuUtils::select_no_help("What do you want to do?", options) {
            Ok(selection) => match selection {
                Selection::LoadProfile => LoadProfileMenu::show(profiles).await,
                Selection::CreateProfile => CreateProfileMenu::show(profiles).await,
                Selection::DeleteProfile => DeleteProfileMenu::show(profiles).await,
                Selection::Exit => MenuResult::Exit,
            },
            Err(error) => {
                debug!("{}", error);
                MenuResult::Exit
            }
        }
    }
}
