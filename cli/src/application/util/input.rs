use std::{error::Error, str::FromStr};

use color_eyre::eyre::{eyre, Result};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};
use tui_textarea::{Input, TextArea};

type Validator = dyn Fn(&str) -> Result<()> + Send + Sync + 'static;

pub struct SimpleTextArea<'a> {
    inner: TextArea<'a>,
    validate: Box<Validator>,
    valid: bool,
    selected: bool,
}

impl SimpleTextArea<'_> {
    fn new_internal<F>(
        placeholder: &str,
        mask_char: Option<char>,
        validate: F,
        selected: bool,
    ) -> Self
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text(placeholder);
        if let Some(mask) = mask_char {
            textarea.set_mask_char(mask);
        }

        let mut instance = Self {
            inner: textarea,
            validate: Box::new(validate),
            valid: false,
            selected,
        };
        instance.update();
        instance
    }

    pub fn new<F>(placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(placeholder, None, validate, false)
    }

    pub fn new_selected<F>(placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(placeholder, None, validate, true)
    }

    pub fn new_password<F>(placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(placeholder, Some('•'), validate, false)
    }

    pub fn new_password_selected<F>(placeholder: &str, validate: F) -> Self
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        Self::new_internal(placeholder, Some('•'), validate, true)
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
        self.update();
    }

    pub fn update(&mut self) {
        let line = self
            .inner
            .lines()
            .first()
            .expect("TextArea should always have at least one line");

        match (self.validate)(line) {
            Ok(()) => self.apply_style(true, "OK".to_string()),
            Err(err) => self.apply_style(false, format!("ERROR: {err}")),
        }
    }

    fn apply_style(&mut self, is_valid: bool, title: String) {
        let (foreground, border) = if is_valid {
            if self.selected {
                (Color::Green, Color::Green)
            } else {
                (Color::LightGreen, Color::LightGreen)
            }
        } else if self.selected {
            (Color::Red, Color::Red)
        } else {
            (Color::LightRed, Color::LightRed)
        };

        self.inner.set_style(Style::default().fg(foreground));
        self.inner.set_block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border)
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
        self.inner.render(area, buffer);
    }

    pub fn type_validation<T>(line: &str) -> Result<()>
    where
        T: FromStr,
        T::Err: Error + Send + Sync + 'static,
    {
        line.parse::<T>().map(|_| ()).map_err(|error| eyre!(error))
    }

    pub fn not_empty_validation(line: &str) -> Result<()> {
        if line.trim().is_empty() {
            Err(eyre!("The field cannot be empty"))
        } else {
            Ok(())
        }
    }
}
