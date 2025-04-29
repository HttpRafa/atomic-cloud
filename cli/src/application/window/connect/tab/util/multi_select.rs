use std::fmt::{Display, Formatter};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::WHITE, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, ListItem, Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    util::{center::CenterWarning, list::ActionList, OK_SELECTED_COLOR, TEXT_FG_COLOR},
    window::{StackBatcher, Window},
    State,
};

type Callback<T> =
    Box<dyn FnOnce(Vec<T>, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static>;

struct Selectable<T: Display> {
    selected: bool,
    inner: T,
}

pub struct MultiSelectWindow<'a, T: Display> {
    /* Callback */
    callback: Option<Callback<T>>,

    /* Window */
    title: &'a str,
    list: ActionList<'a, Selectable<T>>,
}

impl<'a, T: Display> MultiSelectWindow<'a, T> {
    pub fn new<V, F>(title: &'a str, items: V, callback: F) -> Self
    where
        V: IntoIterator<Item = T>,
        F: FnOnce(Vec<T>, &mut StackBatcher, &mut State) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            title,
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
impl<T: Display + Sync + Send> Window for MultiSelectWindow<'_, T> {
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
                KeyCode::Right | KeyCode::Left => {
                    if let Some(selected) = self.list.selected_mut() {
                        selected.selected = !selected.selected;
                    }
                }
                KeyCode::Enter => {
                    let items = self
                        .list
                        .take_items()
                        .into_iter()
                        .filter_map(|item| {
                            if item.selected {
                                Some(item.inner)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    stack.pop();
                    if let Some(callback) = self.callback.take() {
                        callback(items, stack, state)?;
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

impl<T: Display> Widget for &mut MultiSelectWindow<'_, T> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        MultiSelectWindow::<T>::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl<T: Display> MultiSelectWindow<'_, T> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ⇄ to select/deselect, ↵ to confirm, Esc to close.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [title_area, area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);

        Paragraph::new(self.title)
            .centered()
            .white()
            .bold()
            .render(title_area, buffer);

        let block = Block::default()
            .border_style(Style::default().not_bold().fg(WHITE))
            .borders(Borders::BOTTOM);
        block.render(title_area, buffer);

        self.list.render(area, buffer);
        if self.list.is_empty() {
            CenterWarning::render("You dont have any options. Use Esc to close.", area, buffer);
        }
    }
}

impl<T: Display> From<&Selectable<T>> for ListItem<'_> {
    fn from(value: &Selectable<T>) -> Self {
        let (text, color) = if value.selected {
            (format!(" ✓ {}", value.inner), OK_SELECTED_COLOR)
        } else {
            (format!(" ☐ {}", value.inner), TEXT_FG_COLOR)
        };
        ListItem::new(Line::styled(text, color))
    }
}

impl<T: Display> Display for Selectable<T> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.inner)
    }
}
