use getset::Getters;
use serde::Deserialize;

pub mod manager;

pub struct Template {}

#[derive(Deserialize, Getters)]
pub struct PlatformScript {
    #[getset(get = "pub")]
    unix: Option<Script>,
    #[getset(get = "pub")]
    windows: Option<Script>,
}

#[derive(Deserialize, Getters)]
pub struct Script {
    #[getset(get = "pub")]
    command: String,
    #[getset(get = "pub")]
    args: Vec<String>,
}