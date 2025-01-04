use crate::application::profile::Profiles;

mod create_profile;
mod delete_profile;
mod load_profile;
pub mod start;

pub enum MenuResult {
    Success,
    Failed,
}

pub trait Menu {
    fn show(profiles: &mut Profiles) -> MenuResult;
}
