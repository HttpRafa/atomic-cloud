use std::{sync::Arc, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream};
use ratatui::{DefaultTerminal, Frame};
use tokio::{select, time::interval};
use tokio_stream::StreamExt;
use window::{start::StartWindow, StackAction, WindowStack};

mod window;

pub const TICK_RATE: u64 = 4;
pub const FRAME_RATE: u64 = 20;

pub struct Cli {
    running: bool,
    shared: Arc<Shared>,

    stack: WindowStack,
}

pub struct Shared {}

impl Cli {
    pub fn new() -> Self {
        let shared = Shared {};
        Self {
            running: true,
            shared: Arc::new(shared),
            stack: WindowStack::new(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // Push the home window to the stack
        self.stack.push(&self.shared, Box::new(StartWindow));

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
        if let Some(window) = self.stack.current() {
            match window.tick(&self.shared).await? {
                StackAction::Push(window) => {
                    self.stack.push(&self.shared, window);
                }
                StackAction::Pop => {
                    self.stack.pop();
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<()> {
        if let Some(window) = self.stack.current() {
            match window.handle_event(event).await? {
                StackAction::Push(window) => {
                    self.stack.push(&self.shared, window);
                }
                StackAction::Pop => {
                    self.stack.pop();
                }
                _ => {}
            }
        }
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
