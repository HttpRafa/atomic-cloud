use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, StatefulWidget},
};

use super::{ALT_ROW_BG_COLOR, HEADER_STYLE, NORMAL_ROW_BG, SELECTED_STYLE};

pub struct ActionList<T> {
    items: Vec<T>,
    state: ListState,
}

impl<T> ActionList<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    pub fn next(&mut self) {
        self.state.select_next();
    }

    pub fn previous(&mut self) {
        self.state.select_previous();
    }

    pub fn selected(&self) -> Option<&T> {
        self.items.get(self.state.selected()?)
    }
}

impl<T> ActionList<T>
where
    for<'a> ListItem<'a>: From<&'a T>,
{
    pub fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let color = if i % 2 == 0 {
                    NORMAL_ROW_BG
                } else {
                    ALT_ROW_BG_COLOR
                };
                ListItem::from(item).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buffer, &mut self.state);
    }
}
