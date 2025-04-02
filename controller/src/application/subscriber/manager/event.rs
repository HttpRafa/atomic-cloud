use getset::Getters;

use crate::application::{node::Allocation, server::NameAndUuid};

#[derive(Getters, Clone)]
pub struct ServerEvent {
    #[getset(get = "pub")]
    pub id: NameAndUuid,
    #[getset(get = "pub")]
    pub group: Option<String>,
    #[getset(get = "pub")]
    pub allocation: Allocation,
    #[getset(get = "pub")]
    pub token: String,
}
