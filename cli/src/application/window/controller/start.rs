use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
    Frame,
};
use tonic::async_trait;

use crate::application::{
    network::connection::EstablishedConnection,
    window::{StackBatcher, Window, WindowUtils},
    State,
};

pub struct StartWindow {
    /* Connection */
    connection: EstablishedConnection,
}

impl StartWindow {
    pub fn new(connection: EstablishedConnection) -> Self {
        Self { connection }
    }
}

#[async_trait]
impl Window for StartWindow {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        _state: &mut State,
        event: Event,
    ) -> Result<()> {
        if let Event::Key(event) = event {
            if event.code == KeyCode::Esc {
                stack.pops(2);
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &mut StartWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Create areas for header, main, and footer
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header("Controller", header_area, buffer);
        StartWindow::render_footer(footer_area, buffer);
    }
}

impl StartWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to go back.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {}
}
