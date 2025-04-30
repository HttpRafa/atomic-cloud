use getset::Getters;

use crate::application::{
    node::Allocation,
    server::{NameAndUuid, Server},
};

pub type ServerReadyEvent = (ServerEvent, bool);

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

impl From<&Server> for ServerEvent {
    fn from(value: &Server) -> Self {
        Self {
            id: value.id().clone(),
            group: value.group().clone(),
            allocation: value.allocation().clone(),
            token: value.token().clone(),
        }
    }
}
