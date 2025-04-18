use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct SimpleButton<'a> {
    title: &'a str,
    text: &'a str,
    color: (Color, Color),

    selected: bool,
}

impl<'a> SimpleButton<'a> {
    fn new_internal(title: &'a str, text: &'a str, color: (Color, Color), selected: bool) -> Self {
        Self {
            title,
            text,
            color,
            selected,
        }
    }

    pub fn new(title: &'a str, text: &'a str, color: (Color, Color)) -> Self {
        Self::new_internal(title, text, color, false)
    }

    pub fn new_selected(title: &'a str, text: &'a str, color: (Color, Color)) -> Self {
        Self::new_internal(title, text, color, true)
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let color = if self.selected {
            self.color.0
        } else {
            self.color.1
        };

        Paragraph::new(self.text)
            .block(
                Block::default()
                    .style(Style::default().fg(color))
                    .borders(Borders::ALL)
                    .border_style(color)
                    .title(self.title),
            )
            .render(area, buffer);
    }
}
