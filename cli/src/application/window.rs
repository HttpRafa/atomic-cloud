use color_eyre::eyre::Result;
use crossterm::event::Event;
use ratatui::Frame;
use tonic::async_trait;

use super::State;

pub mod start;

pub struct WindowStack(Vec<Box<dyn Window>>);

impl WindowStack {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn current(&mut self) -> Option<&mut Box<dyn Window>> {
        self.0.last_mut()
    }

    pub async fn push(&mut self, state: &mut State, mut window: Box<dyn Window>) -> Result<()> {
        window.init(state).await?;
        self.0.push(window);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<Box<dyn Window>> {
        self.0.pop()
    }
}

pub enum StackAction {
    Nothing,
    Push(Box<dyn Window>),
    Pop,
}

#[async_trait]
pub trait Window {
    async fn init(&mut self, state: &mut State) -> Result<()>;
    async fn tick(&mut self, state: &mut State) -> Result<StackAction>;
    async fn handle_event(&mut self, event: Event) -> Result<StackAction>;

    fn render(&mut self, frame: &mut Frame);
}
