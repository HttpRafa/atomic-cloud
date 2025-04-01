use anyhow::Result;
use getset::Getters;
use simplelog::debug;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use uuid::Uuid;

use crate::network::client::{ChannelMsg, PowerEventMsg, TransferMsg};

use super::{dispatcher::Watcher, Subscriber};

#[derive(Getters)]
pub struct SubscriberManager {
    /* Client */
    #[getset(get = "pub")]
    transfer: Watcher<Uuid, TransferMsg>,
    #[getset(get = "pub")]
    channel: Watcher<String, ChannelMsg>,

    /* Events */
    #[getset(get = "pub")]
    power: Watcher<(), PowerEventMsg>,
}

impl SubscriberManager {
    pub fn init() -> Self {
        Self {
            transfer: Watcher::new(),
            channel: Watcher::new(),
            
            power: Watcher::new(),
        }
    }
}

// Ticking
impl SubscriberManager {
    pub async fn tick(&self) -> Result<()> {
        // Cleanup dead subscribers
        self.channel.cleanup().await;
        self.transfer.cleanup().await;

        self.power.cleanup().await;
        Ok(())
    }
}
