use crate::network::unit::proto::channel_management::ChannelMessageValue;

use super::Event;

#[derive(Debug)]
pub struct ChannelMessageSended {
    pub message: ChannelMessageValue,
}

impl Event for ChannelMessageSended {}
