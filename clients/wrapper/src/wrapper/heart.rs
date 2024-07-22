use log::error;
use std::{sync::Arc, time::{Duration, Instant}};

use super::network::CloudConnection;

pub struct Heart {
    /* Timings */
    pub interval: Duration,
    pub last_beat: Instant,

    /* Network */
    connection: Arc<CloudConnection>,
}

impl Heart {
    pub fn new(interval: Duration, connection: Arc<CloudConnection>) -> Self {
        Self {
            interval,
            last_beat: Instant::now(),
            connection,
        }
    }

    pub async fn tick(&mut self) {
        if self.last_beat.elapsed() > self.interval {
            self.beat().await;
        }
    }

    pub async fn beat(&mut self) {
        self.last_beat = Instant::now();
        if let Err(error) = self.connection.beat_heart().await {
            error!("Failed to report health to controller: {}", error);
        }
    }
}
