use crate::network::server::proto::ChannelMessageValue;

use super::Event;

#[derive(Debug)]
pub struct ChannelMessageSended {
    pub message: ChannelMessageValue,
}

impl Event for ChannelMessageSended {}
