use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Paragraph, Widget},
};

use super::{ERROR_COLOR, OK_COLOR, TEXT_FG_COLOR};

const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];

pub enum Status {
    Loading,
    Error,
    Ok,
    Finished,
}

pub struct StatusDisplay {
    status: Status,
    index: usize,
    message: String,
}

impl StatusDisplay {
    pub fn new(status: Status, message: &str) -> Self {
        Self {
            status,
            index: 0,
            message: message.to_owned(),
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % FRAMES.len();
    }

    pub fn change(&mut self, status: Status, message: &str) {
        self.status = status;
        message.clone_into(&mut self.message);
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status, Status::Loading)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, Status::Finished)
    }

    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let frame = match self.status {
            Status::Loading => FRAMES[self.index],
            Status::Error => "✗",
            Status::Ok | Status::Finished => "✓",
        };
        Paragraph::new(format!("{frame} {}", self.message))
            .fg(match self.status {
                Status::Loading => TEXT_FG_COLOR,
                Status::Error => ERROR_COLOR,
                Status::Ok | Status::Finished => OK_COLOR,
            })
            .bold()
            .centered()
            .render(area, buffer);
    }
}
