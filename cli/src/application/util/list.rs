use std::{fmt::Display, mem};

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
    view: Vec<usize>,

    state: ListState,
}

impl<T: Display> ActionList<'_, T> {
    pub fn new(mut items: Vec<T>, sort: bool) -> Self {
        let mut search = TextArea::default();
        search.set_cursor_line_style(Style::default());
        search.set_placeholder_text("Type to search");

        let mut state = ListState::default();
        state.select_first();

        // Sort the items by name
        if sort {
            items.sort_by_key(std::string::ToString::to_string);
        }

        let mut list = Self {
            search,
            items,
            view: vec![],
            state,
        };
        list.update();
        list
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn previous(&mut self) {
        self.state.select_previous();
    }

    pub fn take_items(&mut self) -> Vec<T> {
        let items = mem::take(&mut self.items);
        self.view.clear();
        self.state.select(None);
        items
    }

    pub fn take_selected(&mut self) -> Option<T> {
        let value =
            self.items
                .remove(*self.view.get(self.state.selected()?).expect(
                    "This should not fail because we update the view every time the user types",
                ));
        self.update();
        Some(value)
    }

    pub fn selected(&self) -> Option<&T> {
        self.view
            .get(self.state.selected()?)
            .and_then(|index| self.items.get(*index))
    }

    pub fn selected_mut(&mut self) -> Option<&mut T> {
        self.view
            .get(self.state.selected()?)
            .and_then(|index| self.items.get_mut(*index))
    }

    fn update(&mut self) {
        self.view.clear();
        self.view
            .extend(self.items.iter().enumerate().filter_map(|(index, item)| {
                if item.to_string().to_lowercase().trim().contains(
                    self.search
                        .lines()
                        .first()
                        .expect("Should always return min one line")
                        .to_lowercase()
                        .trim(),
                ) {
                    Some(index)
                } else {
                    None
                }
            }));
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get_items(&self) -> &[T] {
        &self.items
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
                    self.update();
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
            .view
            .iter()
            .map(|index| {
                let color = if index % 2 == 0 {
                    NORMAL_ROW_BG
                } else {
                    ALT_ROW_BG_COLOR
                };
                let item = self.items.get(*index).expect(
                    "This should not fail because we update the view every time the user types",
                );
                ListItem::from(item).bg(color)
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
