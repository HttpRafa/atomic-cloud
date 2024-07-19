use log::error;
use std::time::{Duration, Instant};

use super::network::CloudConnection;

pub struct Heart {
    pub interval: Duration,
    pub last_beat: Instant,
}

impl Heart {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_beat: Instant::now(),
        }
    }

    pub async fn tick(&mut self, connnection: &mut CloudConnection) {
        if self.last_beat.elapsed() > self.interval {
            self.beat(connnection).await;
        }
    }

    pub async fn beat(&mut self, connnection: &mut CloudConnection) {
        self.last_beat = Instant::now();
        if let Err(error) = connnection.beat_heart().await {
            error!("Failed to report health to controller: {}", error);
        }
    }
}
