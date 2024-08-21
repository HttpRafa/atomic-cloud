use crate::network::server::proto::ChannelMessage;

use super::Event;

pub struct ChannelMessageSended {
    pub message: ChannelMessage,
}

impl Event for ChannelMessageSended {}
