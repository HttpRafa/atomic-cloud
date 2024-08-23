use crate::network::server::proto::ChannelMessage;

use super::Event;

#[derive(Debug)]
pub struct ChannelMessageSended {
    pub message: ChannelMessage,
}

impl Event for ChannelMessageSended {}
