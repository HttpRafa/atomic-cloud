use std::fmt::Display;

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{ListItem, Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    util::{center::CenterWarning, list::ActionList},
    window::{StackBatcher, Window},
    State,
};

type Callback<T> =
    Box<dyn Fn(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static>;

pub struct SelectWindow<'a, T: Display> {
    /* Callback */
    callback: Callback<T>,

    /* State */
    list: ActionList<'a, T>,
}

impl<T: Display> SelectWindow<'_, T> {
    pub fn new<F>(items: Vec<T>, callback: F) -> Self
    where
        F: Fn(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            callback: Box::new(callback),
            list: ActionList::new(items),
        }
    }
}

#[async_trait]
impl<T: Display + Sync + Send> Window for SelectWindow<'_, T>
where
    for<'b> ListItem<'b>: From<&'b T>,
{
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        state: &mut State,
        event: Event,
    ) -> Result<()> {
        if let Event::Key(event) = event {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => {
                    stack.pop();
                    stack.close_tab();
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.list.take_selected() {
                        (self.callback)(selected, stack, state)?;
                    }
                }
                _ => self.list.handle_event(event),
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl<T: Display> Widget for &mut SelectWindow<'_, T>
where
    for<'b> ListItem<'b>: From<&'b T>,
{
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        SelectWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl<T: Display> SelectWindow<'_, T>
where
    for<'b> ListItem<'b>: From<&'b T>,
{
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        self.list.render(area, buffer);
        if self.list.is_empty() {
            CenterWarning::render("You dont have any options. Use Esc to close.", area, buffer);
        }
    }
}
