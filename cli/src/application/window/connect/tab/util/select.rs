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
    State,
    util::{ERROR_SELECTED_COLOR, TEXT_FG_COLOR, center::CenterWarning, list::ActionList},
    window::{StackBatcher, Window, WindowUtils},
};

type Callback<T> =
    Box<dyn FnOnce(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static>;

struct Selectable<T: Display> {
    selected: bool,
    inner: T,
}

pub struct SelectWindow<'a, T: Display> {
    /* Settings */
    // If the user needs to confirm the selection
    confirmation: bool,

    /* Callback */
    callback: Option<Callback<T>>,

    /* Window */
    title: &'a str,
    list: ActionList<'a, Selectable<T>>,
}

impl<'a, T: Display> SelectWindow<'a, T> {
    pub fn new<V, F>(title: &'a str, items: V, callback: F) -> Self
    where
        V: IntoIterator<Item = T>,
        F: FnOnce(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            title,
            confirmation: false,
            callback: Some(Box::new(callback)),
            list: ActionList::new(
                items
                    .into_iter()
                    .map(|inner| Selectable {
                        selected: false,
                        inner,
                    })
                    .collect(),
                true,
            ),
        }
    }
    pub fn new_with_confirmation<V, F>(title: &'a str, items: V, callback: F) -> Self
    where
        V: IntoIterator<Item = T>,
        F: FnOnce(T, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            title,
            confirmation: true,
            callback: Some(Box::new(callback)),
            list: ActionList::new(
                items
                    .into_iter()
                    .map(|inner| Selectable {
                        selected: false,
                        inner,
                    })
                    .collect(),
                true,
            ),
        }
    }
}

#[async_trait]
impl<T: Display + Sync + Send> Window for SelectWindow<'_, T> {
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
                                    stack.pop();
                                    if let Some(callback) = self.callback.take() {
                                        callback(selected.inner, stack, state)?;
                                    }
                                }
                            } else {
                                // Select item
                                selected.selected = true;
                            }
                        }
                    } else {
                        // No confirmation needed
                        if let Some(selected) = self.list.take_selected() {
                            stack.pop();
                            if let Some(callback) = self.callback.take() {
                                callback(selected.inner, stack, state)?;
                            }
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
        let [title_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_tab_header(self.title, title_area, buffer);
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
