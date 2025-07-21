use std::{sync::Arc, time::Duration};

use simplelog::error;
use tokio::time::{Interval, interval};

use super::network::CloudConnection;

pub struct Heart {
    /* Timings */
    interval: Interval,

    /* Network */
    connection: Arc<CloudConnection>,
}

impl Heart {
    pub fn new(period: Duration, connection: Arc<CloudConnection>) -> Self {
        Self {
            interval: interval(period),
            connection,
        }
    }

    pub async fn wait_for_beat(&mut self) {
        self.interval.tick().await;
    }

    pub async fn beat(&mut self) {
        if let Err(error) = self.connection.beat().await {
            error!("Failed to report health to controller: {}", error);
        }
    }
}
