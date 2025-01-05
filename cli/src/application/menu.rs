mod connection;
mod create_profile;
mod delete_profile;
mod load_profile;
pub mod start;

#[derive(PartialEq)]
pub enum MenuResult {
    Success,
    Aborted,
    Failed,
    Exit,
}
