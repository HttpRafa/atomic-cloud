use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use common::cache::FixedSizeCache;
use futures::FutureExt;
use simplelog::warn;
use tokio::{
    sync::RwLock,
    time::{Interval, MissedTickBehavior, interval},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use uuid::Uuid;

use crate::{
    application::{TICK_RATE, subscriber::Subscriber},
    network::manage::ScreenLines,
};

use super::{BoxedScreen, ScreenPullJoinHandle, ScreenWriteJoinHandle};

const SCREEN_TICK_RATE: u64 = TICK_RATE / 3;
const PASSIVE_SCREEN_TICK_RATE_MULTIPLIER: u64 = 250;

type SubscriberHolder<B> = Vec<Subscriber<B>>;

pub struct ScreenManager {
    screens: RwLock<HashMap<Uuid, ActiveScreen>>,
}

impl ScreenManager {
    pub fn init() -> Self {
        Self {
            screens: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_screen(&self, server: &Uuid, mut screen: BoxedScreen) {
        if !screen.is_supported() {
            if let Err(error) = screen.cleanup().await {
                warn!("Failed to cleanup unsupported screen: {}", error);
            }
            return;
        }

        self.screens
            .write()
            .await
            .insert(*server, ActiveScreen::new(screen));
    }

    pub async fn unregister_screen(&self, server: &Uuid) -> Result<()> {
        if let Some(mut screen) = self.screens.write().await.remove(server) {
            // Before we can drop the screen we have to drop the wasm resources first
            screen.cleanup().await?;
            drop(screen); // Drop the screen
        }

        Ok(())
    }

    pub async fn write(&self, server: &Uuid, data: &[u8]) -> Result<ScreenWriteJoinHandle, Status> {
        let screens = self.screens.read().await;
        let screen = screens.get(server).ok_or(Status::unimplemented(
            "The plugin that handles this screen does not support it",
        ))?;
        Ok(screen.write(data))
    }

    pub async fn subscribe_screen(
        &self,
        server: &Uuid,
    ) -> Result<ReceiverStream<Result<ScreenLines, Status>>, Status> {
        let mut screens = self.screens.write().await;
        let screen = screens.get_mut(server).ok_or(Status::unimplemented(
            "The plugin that handles this screen does not support it",
        ))?;

        let (subscriber, receiver) = Subscriber::create_network();
        screen.push(subscriber).await;
        Ok(receiver)
    }
}

// Ticking
impl ScreenManager {
    pub async fn tick(&self) -> Result<()> {
        for screen in self.screens.write().await.values_mut() {
            screen.tick().await?;
        }
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<()> {
        for (_, mut screen) in self.screens.write().await.drain() {
            // Before we can drop the screen we have to drop the wasm resources first
            screen.cleanup().await?;
            drop(screen); // Drop the screen
        }
        Ok(())
    }
}

struct ActiveScreen {
    // First is for normal ticks and second is for passiv ticks to prevent buffers from overflowing
    intervals: (Interval, Interval),
    screen: BoxedScreen,
    handle: Option<ScreenPullJoinHandle>,
    subscribers: SubscriberHolder<ScreenLines>,
    cache: FixedSizeCache<String>,
}

impl ActiveScreen {
    pub fn new(screen: BoxedScreen) -> Self {
        let mut intervals = (
            interval(Duration::from_millis(1000 / SCREEN_TICK_RATE)),
            interval(Duration::from_millis(
                (1000 / SCREEN_TICK_RATE) * PASSIVE_SCREEN_TICK_RATE_MULTIPLIER,
            )),
        );
        intervals
            .0
            .set_missed_tick_behavior(MissedTickBehavior::Skip);
        intervals
            .1
            .set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self {
            intervals,
            screen,
            handle: None,
            subscribers: vec![],
            cache: FixedSizeCache::new(120),
        }
    }

    pub fn write(&self, data: &[u8]) -> ScreenWriteJoinHandle {
        self.screen.write(data)
    }

    pub async fn push(&mut self, subscriber: Subscriber<ScreenLines>) {
        if self.cache.has_data()
            && !subscriber
                .send_message(ScreenLines {
                    lines: self.cache.clone_items(),
                })
                .await
        {
            warn!("Failed to send initial screen data to subscriber!");
            return;
        }

        self.subscribers.push(subscriber);
    }

    pub async fn tick(&mut self) -> Result<()> {
        if self.intervals.0.tick().now_or_never().is_none() {
            // Skip tick
            return Ok(());
        }

        // Remove all dead subscribers
        self.subscribers.retain(Subscriber::is_alive);

        if self.intervals.1.tick().now_or_never().is_none() && self.subscribers.is_empty() {
            // If no one is watching dont pull and no passiv tick is needed
            return Ok(());
        }

        self.handle = match self.handle.take() {
            Some(handle) if handle.is_finished() => {
                let lines = handle.await?.map_err(Into::<Status>::into);
                match lines {
                    Ok(lines) => {
                        if !lines.is_empty() {
                            self.cache.extend(lines.clone());
                            let lines = ScreenLines { lines };
                            for subscriber in &self.subscribers {
                                subscriber.send_network(Ok(lines.clone())).await;
                            }
                        }
                    }
                    Err(error) => {
                        for subscriber in &self.subscribers {
                            subscriber.send_network(Err(error.clone())).await;
                        }
                    }
                }
                Some(self.screen.pull())
            }
            Some(handle) => Some(handle),
            None => Some(self.screen.pull()),
        };

        Ok(())
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        self.screen.cleanup().await
    }
}
