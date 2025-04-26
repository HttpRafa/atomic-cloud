use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use tokio::time::Instant;

use super::{ERROR_COLOR, OK_COLOR, TEXT_FG_COLOR, WARN_COLOR};

const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[allow(dead_code)]
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
    pub fn new<'a, T>(status: Status, message: T) -> Self where T: Into<Cow<'a, str>> {
        Self {
            status,
            index: 0,
            instant: None,
            message: message.into().into_owned(),
        }
    }

    pub fn _new_with_startpoint<'a, T>(status: Status, message: T) -> Self where T: Into<Cow<'a, str>> {
        Self {
            status,
            index: 0,
            instant: Some(Instant::now()),
            message: message.into().into_owned(),
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % FRAMES.len();
    }

    pub fn change<'a, T>(&mut self, status: Status, message: T) where T: Into<Cow<'a, str>> {
        self.status = status;
        self.instant = None;
        self.message = message.into().into_owned();
    }

    pub fn change_with_startpoint<'a, T>(&mut self, status: Status, message: T) where T: Into<Cow<'a, str>> {
        self.status = status;
        self.instant = Some(Instant::now());
        self.message = message.into().into_owned();
    }

    pub fn is_ok(&self) -> bool {
        matches!(self.status, Status::Ok)
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

    pub fn render_in_center(&self, area: Rect, buffer: &mut Buffer) {
        let [area] = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(area);
        self.render(area, buffer);
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
