use std::sync::Arc;

use anyhow::{anyhow, Result};
use common::error::FancyError;
use futures::FutureExt;
use tokio::sync::{mpsc::Receiver, MutexGuard};
use wasmtime::{component::ResourceAny, AsContextMut, Store};

use crate::application::{
    subscriber::{manager::event::ServerEvent, Subscriber},
    Shared,
};

use super::{
    generated::{self, exports::plugin::system::event::Events},
    PluginState,
};

pub struct PluginListener {
    dropped: bool,

    events: Events,
    instance: ResourceAny,

    /* Events */
    server_start: Option<Receiver<Result<ServerEvent>>>,
    server_stop: Option<Receiver<Result<ServerEvent>>>,
}

impl PluginListener {
    pub fn new(instance: (Events, ResourceAny)) -> Self {
        Self {
            dropped: false,
            events: instance.0,
            instance: instance.1,

            server_start: None,
            server_stop: None,
        }
    }

    pub async fn register(&mut self, shared: &Arc<Shared>) {
        if self.events.contains(Events::SERVER_START) {
            let (subscriber, receiver) = Subscriber::create_plugin();
            shared
                .subscribers
                .server_start()
                .subscribe(subscriber)
                .await;
            self.server_start = Some(receiver);
        }
        if self.events.contains(Events::SERVER_STOP) {
            let (subscriber, receiver) = Subscriber::create_plugin();
            shared.subscribers.server_stop().subscribe(subscriber).await;
            self.server_stop = Some(receiver);
        }
    }

    fn collect_events<T>(event: &mut Option<Receiver<Result<T>>>) -> Vec<T> {
        let mut events = Vec::new();
        if let Some(receiver) = event.as_mut() {
            while let Some(Some(event)) = receiver.recv().now_or_never() {
                match event {
                    Ok(event) => events.push(event),
                    Err(error) => {
                        FancyError::print_fancy(
                            &anyhow!("Failed to receive event: {}", error),
                            false,
                        );
                    }
                }
            }
        }
        events
    }

    pub async fn fire_events(
        &mut self,
        bindings: &Arc<generated::Plugin>,
        store: &mut MutexGuard<'_, Store<PluginState>>,
    ) {
        for event in Self::collect_events(&mut self.server_start) {
            let event = event.into();
            match bindings
                .plugin_system_event()
                .listener()
                .call_server_start(store.as_context_mut(), self.instance, &event)
                .await
            {
                Ok(_) => {}
                Err(error) => {
                    FancyError::print_fancy(
                        &anyhow!("Failed to fire server start event: {}", error),
                        false,
                    );
                }
            }
        }
        for event in Self::collect_events(&mut self.server_stop) {
            let event = event.into();
            match bindings
                .plugin_system_event()
                .listener()
                .call_server_stop(store.as_context_mut(), self.instance, &event)
                .await
            {
                Ok(_) => {}
                Err(error) => {
                    FancyError::print_fancy(
                        &anyhow!("Failed to fire server stop event: {}", error),
                        false,
                    );
                }
            }
        }
    }

    pub async fn cleanup(&mut self, store: impl AsContextMut<Data = PluginState>) -> Result<()> {
        self.instance.resource_drop_async(store).await?;
        self.dropped = true;

        Ok(())
    }
}

impl Drop for PluginListener {
    fn drop(&mut self) {
        if !self.dropped {
            FancyError::print_fancy(
                &anyhow!("Resource was not dropped before being deallocated (memory leak)"),
                false,
            );
        }
    }
}
