use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
    Frame,
};
use tonic::async_trait;
use tui_textarea::{Input, TextArea};

use crate::application::State;

use super::{StackBatcher, Window, WindowUtils};

#[derive(Default)]
pub struct CreateWindow {
    name: TextArea<'static>,
    token: TextArea<'static>,
    url: TextArea<'static>,
}

#[async_trait]
impl Window for CreateWindow {
    async fn init(&mut self, stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn handle_event(&mut self, stack: &mut StackBatcher, event: Event) -> Result<()> {
        if let Event::Key(event) = event {
            if event.code == KeyCode::Esc {
                stack.pop();
            } else if event.code == KeyCode::Up {
            } else if event.code == KeyCode::Down {
            } else {
                self.name.input(Input::from(event));
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &mut CreateWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header("Add a new controller", header_area, buffer);
        CreateWindow::render_footer(footer_area, buffer);
    }
}

impl CreateWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to go back.")
            .centered()
            .render(area, buffer);
    }
}
