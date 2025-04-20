use std::{error::Error, str::FromStr};

use color_eyre::eyre::{eyre, Result};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tui_textarea::{Input, TextArea};

use super::{ERROR_COLOR, ERROR_SELECTED_COLOR, OK_COLOR, OK_SELECTED_COLOR};

#[allow(clippy::type_complexity)]
pub struct SimpleTextArea<'a, D> {
    data: D,
    title: &'a str,
    inner: TextArea<'a>,
    validate: Box<dyn Fn(&str, &mut D) -> Result<()> + Send + Sync + 'static>,
    valid: bool,
    selected: bool,
}

impl<'a, D> SimpleTextArea<'a, D> {
    fn new_internal<F>(
        data: D,
        title: &'a str,
        placeholder: &str,
        mask_char: Option<char>,
        validate: F,
        selected: bool,
    ) -> Self
    where
        F: Fn(&str, &mut D) -> Result<()> + Send + Sync + 'static,
    {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text(placeholder);
        if let Some(mask) = mask_char {
            textarea.set_mask_char(mask);
        }

        let mut instance = Self {
            data,
            title,
            inner: textarea,
            validate: Box::new(validate),
            valid: false,
            selected,
        };
        instance.update();
        instance
    }

    pub fn new<F>(data: D, title: &'a str, placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str, &mut D) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(data, title, placeholder, None, validate, false)
    }

    pub fn new_selected<F>(data: D, title: &'a str, placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str, &mut D) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(data, title, placeholder, None, validate, true)
    }

    pub fn new_password<F>(data: D, title: &'a str, placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str, &mut D) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(data, title, placeholder, Some('•'), validate, false)
    }

    pub fn new_password_selected<F>(data: D, title: &'a str, placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str, &mut D) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(data, title, placeholder, Some('•'), validate, true)
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
        self.update();
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn is_invalid(&self) -> bool {
        !self.valid
    }

    pub fn get_first_line(&self) -> String {
        self.inner
            .lines()
            .first()
            .expect("TextArea should always have at least one line")
            .to_string()
    }

    pub fn update(&mut self) {
        let line = self
            .inner
            .lines()
            .first()
            .expect("TextArea should always have at least one line");

        match (self.validate)(line, &mut self.data) {
            Ok(()) => self.apply_style(true, self.title.to_string()),
            Err(error) => self.apply_style(false, format!("{} - {error}", self.title)),
        }
    }

    fn apply_style(&mut self, is_valid: bool, title: String) {
        let color = if is_valid {
            if self.selected {
                OK_SELECTED_COLOR
            } else {
                OK_COLOR
            }
        } else if self.selected {
            ERROR_SELECTED_COLOR
        } else {
            ERROR_COLOR
        };

        self.inner.set_style(Style::default().fg(color));
        self.inner.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(color)
                .title(title),
        );
        self.valid = is_valid;
    }

    pub fn handle_event(&mut self, input: impl Into<Input>) {
        if self.inner.input(input) {
            self.update();
        }
    }

    pub fn render(&self, area: Rect, buffer: &mut Buffer) {
        let [indicator_area, main_area, _] = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .areas(area);

        if self.selected {
            let [indicator_area] = Layout::vertical([Constraint::Length(1)])
                .flex(Flex::Center)
                .areas(indicator_area);
            Paragraph::new(">")
                .left_aligned()
                .light_blue()
                .bold()
                .render(indicator_area, buffer);
        }

        self.inner.render(main_area, buffer);
    }

    pub fn type_validation<T>(line: &str, _: &mut D) -> Result<()>
    where
        T: FromStr,
        T::Err: Error + Send + Sync + 'static,
    {
        line.parse::<T>().map(|_| ()).map_err(|error| eyre!(error))
    }

    pub fn not_empty_validation(line: &str, _: &mut D) -> Result<()> {
        if line.trim().is_empty() {
            Err(eyre!("The field cannot be empty"))
        } else {
            Ok(())
        }
    }

    pub fn already_exists_validation(line: &str, data: &mut D) -> Result<()>
    where
        D: AsRef<[String]>,
    {
        if line.trim().is_empty() {
            Err(eyre!("The field cannot be empty"))
        } else if data.as_ref().iter().any(|entry| entry == line) {
            Err(eyre!("The field value already exists"))
        } else {
            Ok(())
        }
    }
}
