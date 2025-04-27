use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    util::{button::SimpleButton, ERROR_COLOR, ERROR_SELECTED_COLOR, OK_COLOR, OK_SELECTED_COLOR},
    window::{StackBatcher, Window},
    State,
};

type Callback = Box<dyn Fn(bool, &mut StackBatcher, &mut State) -> Result<()> + Send + 'static>;

pub struct ConfirmWindow<'a> {
    /* Callback */
    callback: Callback,

    /* Window */
    title: &'a str,
    message: &'a str,

    current: bool,
    buttons: (SimpleButton<'a>, SimpleButton<'a>),
}

impl<'a> ConfirmWindow<'a> {
    // When supplying, the title is the first element of the tuple for each button
    // The second element is the text in the button
    pub fn new<F>(
        title: &'a str,
        message: &'a str,
        first: (&'a str, &'a str),
        second: (&'a str, &'a str),
        callback: F,
    ) -> Self
    where
        F: Fn(bool, &mut StackBatcher, &mut State) -> Result<()> + Send + 'static,
    {
        Self {
            title,
            message,
            current: false,
            buttons: (
                SimpleButton::new(first.0, first.1, (OK_SELECTED_COLOR, OK_COLOR)),
                SimpleButton::new(second.0, second.1, (ERROR_SELECTED_COLOR, ERROR_COLOR)),
            ),
            callback: Box::new(callback),
        }
    }
}

#[async_trait]
impl Window for ConfirmWindow<'_> {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        state: &mut State,
        event: Event,
    ) -> Result<()> {
        if let Event::Key(event) = event {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => {
                    stack.pop();
                    (self.callback)(false, stack, state)?;
                }
                KeyCode::Right | KeyCode::Tab => {
                    if self.current {
                        self.current = false;
                    }
                }
                KeyCode::Left => {
                    if !self.current {
                        self.current = true;
                    }
                }
                KeyCode::Enter => {
                    stack.pop();
                    (self.callback)(self.current, stack, state)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut ConfirmWindow<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected button
        self.buttons.0.set_selected(self.current);
        self.buttons.1.set_selected(!self.current);

        // Create areas for main, and footer
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        ConfirmWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl ConfirmWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ⇄ to switch, ↵ to confirm. Use Esc to close (No).")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [title_area, _, message_area, button_area, _] = Layout::vertical([
            Constraint::Length(1), // Title
            Constraint::Length(1),
            Constraint::Fill(4), // Message
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(area);

        Paragraph::new(self.title)
            .blue()
            .bold()
            .centered()
            .render(title_area, buffer);
        Paragraph::new(self.message)
            .cyan()
            .bold()
            .centered()
            .render(message_area, buffer);

        let [_, first_area, _, second_area, _] = Layout::horizontal([
            Constraint::Fill(7),
            Constraint::Fill(6),
            Constraint::Fill(1),
            Constraint::Fill(6),
            Constraint::Fill(7),
        ])
        .areas(button_area);

        self.buttons.0.render(first_area, buffer);
        self.buttons.1.render(second_area, buffer);
    }
}
