

mod create_profile;
mod delete_profile;
mod load_profile;
mod connection;
pub mod start;

#[derive(PartialEq)]
pub enum MenuResult {
    Success,
    Aborted,
    Failed,
    Exit,
}
