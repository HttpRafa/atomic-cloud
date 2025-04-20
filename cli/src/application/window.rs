use color_eyre::eyre::Result;
use crossterm::event::Event;
use futures::FutureExt;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
    Frame,
};
use tonic::async_trait;

use crate::VERSION;

use super::State;

pub mod connect;
pub mod create;
pub mod delete;
pub mod start;
pub mod tls;

pub type BoxedWindow = Box<dyn Window + Send + Sync>;

pub struct WindowStack(Vec<BoxedWindow>);

#[derive(Default)]
pub struct StackBatcher(Vec<StackAction>);

impl StackBatcher {
    pub fn push(&mut self, window: BoxedWindow) {
        self.0.push(StackAction::Push(window));
    }

    pub fn pop(&mut self) {
        self.0.push(StackAction::Pop);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl WindowStack {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn current(&mut self) -> Option<&mut BoxedWindow> {
        self.0.last_mut()
    }

    pub async fn handle_event(&mut self, state: &mut State, event: Event) -> Result<()> {
        if let Some(window) = self.current() {
            let mut batcher = StackBatcher::default();
            window.handle_event(&mut batcher, state, event).await?;
            self.apply(state, batcher).await?;
        }
        Ok(())
    }

    pub async fn tick(&mut self, state: &mut State) -> Result<()> {
        if let Some(window) = self.current() {
            let mut batcher = StackBatcher::default();
            window.tick(&mut batcher, state).await?;
            self.apply(state, batcher).await?;
        }
        Ok(())
    }

    pub async fn push(&mut self, state: &mut State, mut window: BoxedWindow) -> Result<()> {
        let mut batcher = StackBatcher::default();
        window.init(&mut batcher, state).await?;
        self.0.push(window);
        self.apply(state, batcher).await?;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<BoxedWindow> {
        self.0.pop()
    }

    pub async fn apply(&mut self, state: &mut State, batcher: StackBatcher) -> Result<()> {
        if batcher.is_empty() {
            return Ok(());
        }
        for action in batcher.0 {
            if let StackAction::Push(window) = action {
                self.push(state, window).boxed_local().await?;
            } else {
                self.pop();
            }
        }
        Ok(())
    }
}

pub enum StackAction {
    Push(BoxedWindow),
    Pop,
}

#[async_trait]
pub trait Window {
    async fn init(&mut self, stack: &mut StackBatcher, state: &mut State) -> Result<()>;
    async fn tick(&mut self, stack: &mut StackBatcher, state: &mut State) -> Result<()>;
    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        state: &mut State,
        event: Event,
    ) -> Result<()>;

    fn render(&mut self, frame: &mut Frame);
}

pub struct WindowUtils;

impl WindowUtils {
    pub fn render_header(title: &str, area: Rect, buffer: &mut Buffer) {
        let [version_area, title_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
        Paragraph::new(format!("{} - {}", "Atomic Cloud CLI", VERSION))
            .blue()
            .bold()
            .centered()
            .render(version_area, buffer);
        Paragraph::new(title)
            .light_blue()
            .bold()
            .centered()
            .render(title_area, buffer);
    }
}
