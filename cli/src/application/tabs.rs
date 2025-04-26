use std::fmt::{Display, Formatter};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use futures::{future::BoxFuture, FutureExt};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::Palette, Color, Stylize},
    symbols::border::PROPORTIONAL_TALL,
    text::Line,
    widgets::{self, Block, Padding, Paragraph, Widget},
};

use super::{
    util::TEXT_FG_COLOR,
    window::{BoxedWindow, StackAction, StackBatcher, WindowStack, WindowUtils},
    State,
};

pub struct Tab {
    name: String,
    palette: Palette,
    stack: WindowStack,
}

pub struct Tabs {
    title: String,
    current: usize,

    inner: Vec<Tab>,
}

impl Tabs {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            current: 0,
            inner: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }

    pub async fn handle_event(&mut self, state: &mut State, event: Event) -> Result<()> {
        // TODO: Im not sure if i want to do this in here and with the F keys
        if let Event::Key(event) = event
            && event.kind == KeyEventKind::Press
            && let KeyCode::F(key) = event.code
        {
            let key = key as usize - 1;
            if self.inner.len() > key {
                self.current = key;
                return Ok(());
            }
        }

        if let Some(stack) = self.inner.get_mut(self.current) {
            let mut batcher = StackBatcher::default();
            stack.stack.handle_event(state, &mut batcher, event).await?;
            self.apply(state, batcher).await?;
        }
        Ok(())
    }

    pub async fn tick(&mut self, state: &mut State) -> Result<()> {
        if let Some(stack) = self.inner.get_mut(self.current) {
            let mut batcher = StackBatcher::default();
            stack.stack.tick(state, &mut batcher).await?;
            self.apply(state, batcher).await?;
        }
        Ok(())
    }

    pub async fn add_tab(
        &mut self,
        state: &mut State,
        name: String,
        palette: Palette,
        init: BoxedWindow,
    ) -> Result<()> {
        let mut stack = WindowStack::new();
        let mut batcher = StackBatcher::default();
        stack.push(state, &mut batcher, init).await?;
        self.inner.push(Tab {
            name,
            palette,
            stack,
        });
        self.current = self.inner.len() - 1;
        self.apply(state, batcher).await?;
        Ok(())
    }

    pub fn close_current(&mut self) -> Tab {
        let tab = self.inner.remove(self.current);
        if self.current > 0 {
            self.current -= 1;
        }
        tab
    }

    pub fn rename_current(&mut self, name: String) {
        if let Some(tab) = self.inner.get_mut(self.current) {
            tab.name = name;
        }
    }

    pub fn apply<'a>(
        &'a mut self,
        state: &'a mut State,
        batcher: StackBatcher,
    ) -> BoxFuture<'a, Result<()>> {
        async move {
            if batcher.is_empty() {
                return Ok(());
            }
            for action in batcher.0 {
                match action {
                    StackAction::AddTab((name, palette, init)) => {
                        self.add_tab(state, name, palette, init).await?;
                    }
                    StackAction::CloseTab => {
                        self.close_current();
                    }
                    StackAction::RenameTab(name) => {
                        self.rename_current(name);
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        .boxed()
    }
}

impl Widget for &mut Tabs {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header(&self.title, header_area, buffer);
        Tabs::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl Tabs {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use F keys to switch tabs")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let area = WindowUtils::render_background(area, buffer);

        let [tabs_area, inner_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let titles = self
            .inner
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                format!(" {} <F{}> ", tab, i + 1)
                    .fg(TEXT_FG_COLOR)
                    .bg(tab.palette.c900)
                    .into()
            })
            .collect::<Vec<Line<'_>>>();

        if let Some(tab) = self.inner.get_mut(self.current) {
            widgets::Tabs::new(titles)
                .highlight_style((Color::default(), tab.palette.c700))
                .select(self.current)
                .padding("", "")
                .divider(" ")
                .render(tabs_area, buffer);

            let block = Block::bordered()
                .border_set(PROPORTIONAL_TALL)
                .padding(Padding::horizontal(1))
                .border_style(tab.palette.c700);
            let block_area = block.inner(inner_area);
            block.render(inner_area, buffer);

            tab.stack.render(block_area, buffer);
        }
    }
}

impl Display for Tab {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
