use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
};

use super::WARN_SELECTED_COLOR;

pub struct CenterWarning();

impl CenterWarning {
    pub fn render(message: &str, area: Rect, buffer: &mut Buffer) {
        let [area] = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(area);
        Paragraph::new(message)
            .fg(WARN_SELECTED_COLOR)
            .bold()
            .centered()
            .render(area, buffer);
    }
}
