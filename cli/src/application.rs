use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream};
use network::known_host::manager::KnownHosts;
use profile::manager::Profiles;
use ratatui::{DefaultTerminal, Frame};
use tokio::{
    select,
    time::{interval, MissedTickBehavior},
};
use tokio_stream::StreamExt;
use window::{start::StartWindow, tls::TrustTlsWindow, StackBatcher, WindowStack};

mod network;
mod profile;
mod tabs;
mod util;
mod window;

pub const TICK_RATE: u64 = 10;
pub const FRAME_RATE: u64 = 15;

pub struct Cli {
    running: bool,
    state: State,

    stack: WindowStack,
}

pub struct State {
    profiles: Profiles,
    known_hosts: Arc<KnownHosts>,
}

impl Cli {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            running: true,
            state: State {
                profiles: Profiles::load().await?,
                known_hosts: Arc::new(KnownHosts::load().await?),
            },
            stack: WindowStack::new(),
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // Push the home window to the stack
        self.stack
            .push(
                &mut self.state,
                &mut StackBatcher::default(),
                Box::new(StartWindow::new()),
            )
            .await?;

        // Events
        let mut events = EventStream::new();

        // Intervals
        let mut frame_interval = interval(Duration::from_millis(1000 / FRAME_RATE));
        frame_interval.set_missed_tick_behavior(MissedTickBehavior::Burst);
        let mut tick_interval = interval(Duration::from_millis(1000 / TICK_RATE));

        // Main loop
        while self.running {
            select! {
                _ = frame_interval.tick() => {terminal.draw(|frame| self.render(frame))?;},
                _ = tick_interval.tick() => self.tick().await?,
                Some(Ok(event)) = events.next() => self.handle_event(event).await?,
            }
        }
        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        if let Some(request) = self.state.known_hosts.requests.dequeue() {
            self.stack
                .push(
                    &mut self.state,
                    &mut StackBatcher::default(),
                    Box::new(TrustTlsWindow::new(request)),
                )
                .await?;
        }
        self.state.known_hosts.requests.cleanup();

        self.stack
            .tick(&mut self.state, &mut StackBatcher::default())
            .await?;
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<()> {
        self.stack
            .handle_event(&mut self.state, &mut StackBatcher::default(), event)
            .await?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        if !self.stack.render(frame.area(), frame.buffer_mut()) {
            self.running = false;
        }
    }
}
