use std::fmt::Display;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use tui_textarea::{Input, Key, TextArea};

use super::{ALT_ROW_BG_COLOR, NORMAL_ROW_BG, SELECTED_STYLE};

pub struct ActionList<'a, T: Display> {
    search: TextArea<'a>,

    items: Vec<T>,
    state: ListState,
}

impl<T: Display> ActionList<'_, T> {
    pub fn new(items: Vec<T>) -> Self {
        let mut search = TextArea::default();
        search.set_cursor_line_style(Style::default());
        search.set_placeholder_text("Type to search");

        let mut state = ListState::default();
        state.select_first();

        Self {
            search,
            items,
            state,
        }
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn previous(&mut self) {
        self.state.select_previous();
    }

    pub fn selected(&self) -> Option<&T> {
        // Reproduce search and order from displayed list
        self.items
            .iter()
            .filter(|item| {
                item.to_string().to_lowercase().trim().contains(
                    self.search
                        .lines()
                        .first()
                        .expect("Should always return min one line")
                        .to_lowercase()
                        .trim(),
                )
            })
            .nth(self.state.selected()?)
    }

    pub fn selected_mut(&mut self) -> Option<&mut T> {
        // Reproduce search and order from displayed list
        self.items
            .iter_mut()
            .filter(|item| {
                item.to_string().to_lowercase().trim().contains(
                    self.search
                        .lines()
                        .first()
                        .expect("Should always return min one line")
                        .to_lowercase()
                        .trim(),
                )
            })
            .nth(self.state.selected()?)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn handle_event(&mut self, input: impl Into<Input>) {
        let input = input.into();
        match input {
            Input {
                key: Key::Enter, ..
            } => {}
            Input { key: Key::Up, .. } => self.previous(),
            Input { key: Key::Down, .. } => self.next(),
            input => {
                if self.search.input(input) {
                    self.state.select_first();
                }
            }
        }
    }
}

impl<T: Display> ActionList<'_, T>
where
    for<'b> ListItem<'b>: From<&'b T>,
{
    pub fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        let [search_area, main_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                if item.to_string().to_lowercase().trim().contains(
                    self.search
                        .lines()
                        .first()
                        .expect("Should always return min one line")
                        .to_lowercase()
                        .trim(),
                ) {
                    let color = if i % 2 == 0 {
                        NORMAL_ROW_BG
                    } else {
                        ALT_ROW_BG_COLOR
                    };
                    Some(ListItem::from(item).bg(color))
                } else {
                    None
                }
            })
            .collect();

        let list = List::new(items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, main_area, buffer, &mut self.state);

        // Search
        let [symbol_area, main_area] =
            Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)]).areas(search_area);
        Paragraph::new("?")
            .left_aligned()
            .green()
            .bold()
            .render(symbol_area, buffer);
        self.search.render(main_area, buffer);
    }
}
