use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use tokio::time::Instant;

use super::{ERROR_COLOR, OK_COLOR, TEXT_FG_COLOR, WARN_COLOR};

const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];

pub enum Status {
    Loading,
    Error,
    Warn,
    Ok,

    NotPerfect,
    Fatal,
    Successful,
}

pub struct StatusDisplay {
    status: Status,
    index: usize,
    instant: Option<Instant>,
    message: String,
}

impl StatusDisplay {
    pub fn new(status: Status, message: &str) -> Self {
        Self {
            status,
            index: 0,
            instant: None,
            message: message.to_owned(),
        }
    }

    pub fn new_with_startpoint(status: Status, message: &str) -> Self {
        Self {
            status,
            index: 0,
            instant: Some(Instant::now()),
            message: message.to_owned(),
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % FRAMES.len();
    }

    pub fn change(&mut self, status: Status, message: &str) {
        self.status = status;
        self.instant = None;
        message.clone_into(&mut self.message);
    }

    pub fn change_with_startpoint(&mut self, status: Status, message: &str) {
        self.status = status;
        self.instant = Some(Instant::now());
        message.clone_into(&mut self.message);
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.status, Status::Loading)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, Status::Successful)
            || matches!(self.status, Status::Fatal)
            || matches!(self.status, Status::NotPerfect)
    }

    pub fn is_fatal(&self) -> bool {
        matches!(self.status, Status::Fatal)
    }

    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let frame = match self.status {
            Status::Loading => FRAMES[self.index],
            Status::Error | Status::Fatal => "✗",
            Status::Warn | Status::NotPerfect => "⚠",
            Status::Ok | Status::Successful => "✓",
        };
        let text = if let Some(instant) = &self.instant {
            format!(
                "{frame} {} ({:.2}s)",
                self.message,
                instant.elapsed().as_secs_f32()
            )
        } else {
            format!("{frame} {}", self.message)
        };
        Paragraph::new(text)
            .fg(match self.status {
                Status::Loading => TEXT_FG_COLOR,
                Status::Error | Status::Fatal => ERROR_COLOR,
                Status::Warn | Status::NotPerfect => WARN_COLOR,
                Status::Ok | Status::Successful => OK_COLOR,
            })
            .bold()
            .centered()
            .render(area, buffer);
    }
}
