use color_eyre::eyre::Result;
use crossterm::event::Event;
use futures::{future::BoxFuture, FutureExt};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::Palette, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tonic::async_trait;

use crate::VERSION;

use super::{
    util::{HEADER_STYLE, NORMAL_ROW_BG},
    State,
};

pub mod connect;
pub mod controller;
pub mod create;
pub mod delete;
pub mod start;
pub mod tls;

pub type BoxedWindow = Box<dyn Window + Send + Sync>;

pub struct WindowStack(Vec<BoxedWindow>);

#[derive(Default)]
pub struct StackBatcher(pub Vec<StackAction>);

impl StackBatcher {
    pub fn add(&mut self, action: StackAction) {
        self.0.push(action);
    }

    pub fn push(&mut self, window: BoxedWindow) {
        self.0.push(StackAction::Push(window));
    }

    pub fn add_tab(&mut self, name: &str, palette: Palette, init: BoxedWindow) {
        self.0
            .push(StackAction::AddTab((name.to_owned(), palette, init)));
    }

    pub fn pop(&mut self) {
        self.0.push(StackAction::Pop);
    }

    pub fn pops(&mut self, amount: usize) {
        for _ in 0..amount {
            self.pop();
        }
    }

    pub fn close_tab(&mut self) {
        self.0.push(StackAction::CloseTab);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl WindowStack {
    pub fn new() -> Self {
        Self(vec![])
    }

    fn current(&mut self) -> Option<&mut BoxedWindow> {
        self.0.last_mut()
    }

    pub fn render(&mut self, area: Rect, buffer: &mut Buffer) -> bool {
        if let Some(window) = self.current() {
            window.render(area, buffer);
            true
        } else {
            false
        }
    }

    pub async fn handle_event(
        &mut self,
        state: &mut State,
        upper: &mut StackBatcher,
        event: Event,
    ) -> Result<()> {
        if let Some(window) = self.current() {
            let mut batcher = StackBatcher::default();
            window.handle_event(&mut batcher, state, event).await?;
            self.apply(state, upper, batcher).await?;
        }
        Ok(())
    }

    pub async fn tick(&mut self, state: &mut State, upper: &mut StackBatcher) -> Result<()> {
        if let Some(window) = self.current() {
            let mut batcher = StackBatcher::default();
            window.tick(&mut batcher, state).await?;
            self.apply(state, upper, batcher).await?;
        }
        Ok(())
    }

    pub async fn push(
        &mut self,
        state: &mut State,
        upper: &mut StackBatcher,
        mut window: BoxedWindow,
    ) -> Result<()> {
        let mut batcher = StackBatcher::default();
        window.init(&mut batcher, state).await?;
        self.0.push(window);
        self.apply(state, upper, batcher).await?;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<BoxedWindow> {
        self.0.pop()
    }

    pub fn apply<'a>(
        &'a mut self,
        state: &'a mut State,
        upper: &'a mut StackBatcher,
        batcher: StackBatcher,
    ) -> BoxFuture<'a, Result<()>> {
        async move {
            if batcher.is_empty() {
                return Ok(());
            }
            for action in batcher.0 {
                match action {
                    StackAction::Push(window) => {
                        self.push(state, upper, window).await?;
                    }
                    StackAction::Pop => {
                        self.pop();
                    }
                    action => upper.add(action),
                }
            }
            Ok(())
        }
        .boxed()
    }
}

pub enum StackAction {
    Push(BoxedWindow),
    Pop,
    AddTab((String, Palette, BoxedWindow)),
    CloseTab,
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

    fn render(&mut self, area: Rect, buffer: &mut Buffer);
}

pub struct WindowUtils;

impl WindowUtils {
    pub fn render_header(title: &str, area: Rect, buffer: &mut Buffer) {
        let [version_area, title_area] =
            Layout::vertical([Constraint::Min(1), Constraint::Fill(1)]).areas(area);
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

    pub fn render_background(area: Rect, buffer: &mut Buffer) -> Rect {
        let block = Block::new()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);
        let inner_area = block.inner(area);
        block.render(area, buffer);
        inner_area
    }
}
