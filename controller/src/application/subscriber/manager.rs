use anyhow::Result;
use event::ServerEvent;
use getset::Getters;
use uuid::Uuid;

use crate::network::client::{ChannelMsg, PowerMsg, TransferMsg};

use super::watcher::Watcher;

pub mod event;

#[derive(Getters)]
pub struct PluginEvents {
    #[getset(get = "pub")]
    server_start: Watcher<(), ServerEvent>,
    #[getset(get = "pub")]
    server_stop: Watcher<(), ServerEvent>,
}

#[derive(Getters)]
pub struct NetworkEvents {
    /* Client */
    #[getset(get = "pub")]
    transfer: Watcher<Uuid, TransferMsg>,
    #[getset(get = "pub")]
    channel: Watcher<String, ChannelMsg>,

    #[getset(get = "pub")]
    power: Watcher<(), PowerMsg>,
}

#[derive(Getters)]
pub struct SubscriberManager {
    /* Events */
    #[getset(get = "pub")]
    plugin: PluginEvents,
    #[getset(get = "pub")]
    network: NetworkEvents,
}

impl SubscriberManager {
    pub fn init() -> Self {
        Self {
            plugin: PluginEvents {
                server_start: Watcher::new(),
                server_stop: Watcher::new(),
            },
            network: NetworkEvents {
                transfer: Watcher::new(),
                channel: Watcher::new(),
                power: Watcher::new(),
            },
        }
    }
}

// Ticking
impl SubscriberManager {
    pub async fn tick(&self) -> Result<()> {
        // Cleanup dead subscribers
        self.network.channel.cleanup().await;
        self.network.transfer.cleanup().await;
        self.network.power.cleanup().await;

        self.plugin.server_start.cleanup().await;
        self.plugin.server_stop.cleanup().await;
        Ok(())
    }
}
