use std::fmt::{Display, Formatter};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{ListItem, Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    util::{center::CenterWarning, list::ActionList, ERROR_SELECTED_COLOR, TEXT_FG_COLOR},
    window::{StackBatcher, Window},
    State,
};

type Callback<T> =
    Box<dyn Fn(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static>;

struct Selectable<T: Display> {
    selected: bool,
    inner: T,
}

pub struct SelectWindow<'a, T: Display> {
    /* Settings */
    // If the user needs to confirm the selection
    confirmation: bool,

    /* Callback */
    callback: Callback<T>,

    /* State */
    list: ActionList<'a, Selectable<T>>,
}

impl<T: Display> SelectWindow<'_, T> {
    pub fn new<F>(items: Vec<T>, callback: F) -> Self
    where
        F: Fn(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            confirmation: false,
            callback: Box::new(callback),
            list: ActionList::new(
                items
                    .into_iter()
                    .map(|inner| Selectable {
                        selected: false,
                        inner,
                    })
                    .collect(),
            ),
        }
    }
    pub fn new_with_confirmation<F>(items: Vec<T>, callback: F) -> Self
    where
        F: Fn(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            confirmation: true,
            callback: Box::new(callback),
            list: ActionList::new(
                items
                    .into_iter()
                    .map(|inner| Selectable {
                        selected: false,
                        inner,
                    })
                    .collect(),
            ),
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
                    if self.confirmation {
                        if let Some(selected) = self.list.selected_mut() {
                            if selected.selected {
                                // Confirmed selection
                                if let Some(selected) = self.list.take_selected() {
                                    (self.callback)(selected.inner, stack, state)?;
                                }
                            } else {
                                // Select item
                                selected.selected = true;
                            }
                        }
                    } else {
                        // No confirmation needed
                        if let Some(selected) = self.list.take_selected() {
                            (self.callback)(selected.inner, stack, state)?;
                        }
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

impl<T: Display> Widget for &mut SelectWindow<'_, T> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        SelectWindow::<T>::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl<T: Display> SelectWindow<'_, T> {
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

impl<T: Display> From<&Selectable<T>> for ListItem<'_> {
    fn from(value: &Selectable<T>) -> Self {
        let (text, color) = if value.selected {
            (format!(" {} (Confirm)", value.inner), ERROR_SELECTED_COLOR)
        } else {
            (format!(" {}", value.inner), TEXT_FG_COLOR)
        };
        ListItem::new(Line::styled(text, color))
    }
}

impl<T: Display> Display for Selectable<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.inner)
    }
}
