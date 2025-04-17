use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use tonic::async_trait;
use url::Url;

use crate::application::{
    util::{input::SimpleTextArea, HEADER_STYLE, NORMAL_ROW_BG},
    State,
};

use super::{StackBatcher, Window, WindowUtils};

pub struct CreateWindow {
    current: Field,

    name: SimpleTextArea<'static>,
    token: SimpleTextArea<'static>,
    url: SimpleTextArea<'static>,
}

enum Field {
    Name,
    Token,
    Url,
}

impl Default for CreateWindow {
    fn default() -> Self {
        Self {
            current: Field::Name,
            name: SimpleTextArea::new_selected(
                "Please enter the name of the controller",
                SimpleTextArea::not_empty_validation,
            ),
            token: SimpleTextArea::new_password(
                "Please enter the token required to access the controller",
                SimpleTextArea::not_empty_validation,
            ),
            url: SimpleTextArea::new(
                "Please enter the url of the controller",
                SimpleTextArea::type_validation::<Url>,
            ),
        }
    }
}

#[async_trait]
impl Window for CreateWindow {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn handle_event(&mut self, stack: &mut StackBatcher, event: Event) -> Result<()> {
        if let Event::Key(event) = event {
            match event.code {
                KeyCode::Esc => stack.pop(),
                KeyCode::Up => match self.current {
                    Field::Name => {}
                    Field::Token => {
                        self.current = Field::Name;
                        self.token.set_selected(false);
                        self.name.set_selected(true);
                    }
                    Field::Url => {
                        self.current = Field::Token;
                        self.url.set_selected(false);
                        self.token.set_selected(true);
                    }
                },
                KeyCode::Down => match self.current {
                    Field::Name => {
                        self.current = Field::Token;
                        self.name.set_selected(false);
                        self.token.set_selected(true);
                    }
                    Field::Token => {
                        self.current = Field::Url;
                        self.token.set_selected(false);
                        self.url.set_selected(true);
                    }
                    Field::Url => {}
                },
                KeyCode::Enter => {}
                _ => match self.current {
                    Field::Name => self.name.handle_event(event),
                    Field::Token => self.token.handle_event(event),
                    Field::Url => self.url.handle_event(event),
                },
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

        let block = Block::new()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);
        block.render(main_area, buffer);

        let [_, name_area, token_area, url_area] = Layout::vertical([
            Constraint::Length(1), // Empty space
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(main_area);

        self.name.render(name_area, buffer);
        self.token.render(token_area, buffer);
        self.url.render(url_area, buffer);
    }
}

impl CreateWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to go back.")
            .centered()
            .render(area, buffer);
    }
}
