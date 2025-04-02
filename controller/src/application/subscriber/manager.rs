use anyhow::Result;
use event::ServerEvent;
use getset::Getters;
use uuid::Uuid;

use crate::network::client::{ChannelMsg, TransferMsg};

use super::dispatcher::Watcher;

pub mod event;

#[derive(Getters)]
pub struct SubscriberManager {
    /* Client */
    #[getset(get = "pub")]
    transfer: Watcher<Uuid, TransferMsg>,
    #[getset(get = "pub")]
    channel: Watcher<String, ChannelMsg>,

    /* Events */
    #[getset(get = "pub")]
    server_start: Watcher<(), ServerEvent>,
    #[getset(get = "pub")]
    server_stop: Watcher<(), ServerEvent>,
}

impl SubscriberManager {
    pub fn init() -> Self {
        Self {
            transfer: Watcher::new(),
            channel: Watcher::new(),

            server_start: Watcher::new(),
            server_stop: Watcher::new(),
        }
    }
}

// Ticking
impl SubscriberManager {
    pub async fn tick(&self) -> Result<()> {
        // Cleanup dead subscribers
        self.channel.cleanup().await;
        self.transfer.cleanup().await;

        self.server_start.cleanup().await;
        self.server_stop.cleanup().await;
        Ok(())
    }
}
