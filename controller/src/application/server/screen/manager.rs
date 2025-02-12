use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use common::cache::FixedSizeCache;
use futures::FutureExt;
use simplelog::warn;
use tokio::{
    sync::RwLock,
    time::{interval, Interval, MissedTickBehavior},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use uuid::Uuid;

use crate::{
    application::{plugin::BoxedScreen, subscriber::Subscriber},
    network::manage::ScreenLines,
};

use super::ScreenJoinHandle;

const SCREEN_TICK_RATE: u64 = 2;

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

    pub async fn register_screen(&self, server: &Uuid, screen: BoxedScreen) {
        if !screen.is_supported() {
            return;
        }

        self.screens
            .write()
            .await
            .insert(*server, ActiveScreen::new(screen));
    }

    pub async fn unregister_screen(&self, server: &Uuid) {
        self.screens.write().await.remove(server);
    }

    pub async fn subscribe_screen(
        &self,
        server: &Uuid,
    ) -> Result<ReceiverStream<Result<ScreenLines, Status>>, Status> {
        let mut screens = self.screens.write().await;
        let screen = screens.get_mut(server).ok_or(Status::unimplemented(
            "The plugin that handles this screen does not support it",
        ))?;

        let (subscriber, receiver) = Subscriber::create();
        screen.push(subscriber).await;
        Ok(receiver)
    }
}

// Ticking
impl ScreenManager {
    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub async fn tick(&self) -> Result<()> {
        for screen in self.screens.write().await.values_mut() {
            screen.tick().await?;
        }
        Ok(())
    }
}

struct ActiveScreen {
    interval: Interval,
    screen: BoxedScreen,
    handle: Option<ScreenJoinHandle>,
    subscribers: SubscriberHolder<ScreenLines>,
    cache: FixedSizeCache<String>,
}

impl ActiveScreen {
    pub fn new(screen: BoxedScreen) -> Self {
        let mut interval = interval(Duration::from_millis(1000 / SCREEN_TICK_RATE));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self {
            interval,
            screen,
            handle: None,
            subscribers: vec![],
            cache: FixedSizeCache::new(91),
        }
    }

    pub async fn push(&mut self, subscriber: Subscriber<ScreenLines>) {
        if self.cache.has_data() {
            if let Err(error) = subscriber
                .0
                .send(Ok(ScreenLines {
                    lines: self.cache.clone_items(),
                }))
                .await
            {
                warn!(
                    "Failed to send initial screen data to subscriber: {}",
                    error
                );
                return;
            }
        }

        self.subscribers.push(subscriber);
    }

    pub async fn tick(&mut self) -> Result<()> {
        if self.interval.tick().now_or_never().is_none() {
            // Skip tick
            return Ok(());
        }

        // Remove all dead subscribers
        self.subscribers.retain(Subscriber::is_alive);

        if self.subscribers.is_empty() {
            // If no one is watching dont pull
            return Ok(());
        }

        self.handle = match self.handle.take() {
            Some(handle) if handle.is_finished() => {
                let lines = handle.await?.map_err(Into::into);
                {
                    let lines = lines.clone().map(|lines| ScreenLines { lines });
                    for subscriber in &self.subscribers {
                        subscriber.0.send(lines.clone()).await?;
                    }
                }
                if let Ok(lines) = lines {
                    self.cache.extend(lines);
                }
                None
            }
            Some(handle) => Some(handle),
            None => Some(self.screen.pull()),
        };

        Ok(())
    }
}
