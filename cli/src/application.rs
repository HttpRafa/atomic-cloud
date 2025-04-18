use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream};
use network::known_host::manager::KnownHosts;
use profile::manager::Profiles;
use ratatui::{DefaultTerminal, Frame};
use tokio::{select, spawn, time::interval};
use tokio_stream::StreamExt;
use window::{start::StartWindow, tls::TrustTlsWindow, WindowStack};

mod network;
mod profile;
mod util;
mod window;

pub const TICK_RATE: u64 = 5;
pub const FRAME_RATE: u64 = 20;

pub struct Cli {
    running: bool,
    state: State,

    stack: WindowStack,
}

pub struct State {
    ticks: u64,

    profiles: Profiles,
    known_hosts: Arc<KnownHosts>,
}

impl Cli {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            running: true,
            state: State {
                ticks: 0,

                profiles: Profiles::load().await?,
                known_hosts: Arc::new(KnownHosts::load().await?),
            },
            stack: WindowStack::new(),
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // Push the home window to the stack
        self.stack
            .push(&mut self.state, Box::new(StartWindow::default()))
            .await?;

        // Events
        let mut events = EventStream::new();

        // Intervals
        let mut frame_interval = interval(Duration::from_millis(1000 / FRAME_RATE));
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
        if let Some(request) = self.state.known_hosts.next().await {
            self.stack
                .push(&mut self.state, Box::new(TrustTlsWindow::new(request)))
                .await?;
        }

        self.state.ticks += 1;
        if self.state.ticks == 5 * TICK_RATE {
            let known = self.state.known_hosts.clone();
            spawn(async move {
                known.is_trusted("test", "test123".as_bytes()).await.expect("Failed to check known hosts");
            });
        }

        self.stack.tick(&mut self.state).await?;
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<()> {
        self.stack.handle_event(&mut self.state, event).await?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        if let Some(window) = self.stack.current() {
            window.render(frame);
        } else {
            self.running = false;
        }
    }
}
