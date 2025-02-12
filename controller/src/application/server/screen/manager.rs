use std::collections::HashMap;

use anyhow::Result;
use tokio::{sync::RwLock, task::JoinHandle};
use uuid::Uuid;

use crate::application::{plugin::BoxedScreen, subscriber::Subscriber};

use super::{cache::ScreenCache, PullError, ScreenMessage};

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
    screen: BoxedScreen,
    handle: Option<JoinHandle<Result<Vec<ScreenMessage>, PullError>>>,
    subscribers: SubscriberHolder<Vec<ScreenMessage>>,
    cache: ScreenCache,
}

impl ActiveScreen {
    pub fn new(screen: BoxedScreen) -> Self {
        Self {
            screen,
            handle: None,
            subscribers: vec![],
            cache: ScreenCache::new(),
        }
    }

    pub async fn tick(&mut self) -> Result<()> {
        // Remove all dead subscribers
        self.subscribers
            .retain(crate::application::subscriber::Subscriber::is_alive);

        if self.subscribers.is_empty() {
            // If no one is watching dont pull
            return Ok(());
        }

        self.handle = match self.handle.take() {
            Some(handle) => {
                if handle.is_finished() {
                    let value = handle.await?.map_err(Into::into);
                    for subscriber in &self.subscribers {
                        subscriber.0.send(value.clone()).await?;
                    }
                    if let Ok(value) = value {
                        for value in value {
                            self.cache.push(value);
                        }
                    }
                    None
                } else {
                    Some(handle)
                }
            }
            None => Some(self.screen.pull()),
        };

        Ok(())
    }
}
