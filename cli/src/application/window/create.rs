use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};
use tonic::async_trait;
use url::Url;

use crate::{
    application::{
        network::connection::task::ConnectTask,
        profile::Profile,
        util::{
            area::SimpleTextArea,
            status::{Status, StatusDisplay},
        },
        State,
    },
    VERSION,
};

use super::{StackBatcher, Window, WindowUtils};

pub struct CreateWindow {
    /* Handles */
    connect: Option<ConnectTask>,

    /* Window */
    status: StatusDisplay,

    current: Field,

    name: SimpleTextArea<'static, Vec<String>>,
    token: SimpleTextArea<'static, ()>,
    url: SimpleTextArea<'static, ()>,
}

enum Field {
    Name,
    Token,
    Url,
}

impl CreateWindow {
    pub fn new(state: &mut State) -> Self {
        Self {
            connect: None,
            status: StatusDisplay::new(Status::Error, ""),
            current: Field::Name,
            name: SimpleTextArea::new_selected(
                state.profiles.get_names(),
                "Name",
                "Please enter the name of the controller",
                SimpleTextArea::already_exists_validation,
            ),
            token: SimpleTextArea::new_password(
                (),
                "Token",
                "Please enter the token required to access the controller",
                SimpleTextArea::not_empty_validation,
            ),
            url: SimpleTextArea::new(
                (),
                "URL",
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

    async fn tick(&mut self, _stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        // Network connection
        if let Some(task) = &mut self.connect {
            match task.get().await {
                Ok(Some(Ok(connection))) => {
                    if let Err(error) = state
                        .profiles
                        .create_profile(&Profile::new(
                            &self.name.get_first_line(),
                            &self.token.get_first_line(),
                            self.url
                                .get_first_line()
                                .parse::<Url>()
                                .expect("Should be validated by the validation process"),
                        ))
                        .await
                    {
                        self.status
                            .change(Status::Fatal, &format!("{}", error.root_cause()));
                    } else if connection.is_incompatible() {
                        self.status.change(
                        Status::NotPerfect,
                        &format!(
                            "Controller created. Press Esc to go back. Warning: your CLI protocol version ({}) doesn't match the server's ({}); some features may not work correctly.",
                            VERSION.protocol,
                            connection.get_protocol(),
                        ),
                    );
                    } else {
                        self.status.change(
                            Status::Successful,
                            "Controller created successfully. Press Esc to go back.",
                        );
                    }
                }
                Err(error) | Ok(Some(Err(error))) => {
                    self.status
                        .change(Status::Fatal, &format!("{}", error.root_cause()));
                }
                _ => {}
            }
        }

        // UI
        self.status.next();
        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        state: &mut State,
        event: Event,
    ) -> Result<()> {
        if !self.status.is_loading()
            && let Event::Key(event) = event
        {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => stack.pop(),
                KeyCode::Up => match self.current {
                    Field::Name => {}
                    Field::Token => self.current = Field::Name,
                    Field::Url => self.current = Field::Token,
                },
                KeyCode::Down | KeyCode::Tab => match self.current {
                    Field::Name => self.current = Field::Token,
                    Field::Token => self.current = Field::Url,
                    Field::Url => {}
                },
                KeyCode::Enter => {
                    if !self.status.is_finished()
                        && self.name.is_valid()
                        && self.token.is_valid()
                        && self.url.is_valid()
                    {
                        self.status.change_with_startpoint(
                            Status::Loading,
                            "Checking if we can reach the controller...",
                        );
                        let profile = Profile::new(
                            &self.name.get_first_line(),
                            &self.token.get_first_line(),
                            self.url
                                .get_first_line()
                                .parse::<Url>()
                                .expect("Should be validated by the validation process"),
                        );
                        self.connect =
                            Some(profile.establish_connection(state.known_hosts.clone()));
                    }
                }
                _ => {
                    if self.status.is_finished() && self.status.is_fatal() {
                        self.status.change(Status::Error, "Please fix your fields");
                    }
                    if !self.status.is_finished() {
                        match self.current {
                            Field::Name => self.name.handle_event(event),
                            Field::Token => self.token.handle_event(event),
                            Field::Url => self.url.handle_event(event),
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut CreateWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected field
        self.name.set_selected(matches!(self.current, Field::Name));
        self.token
            .set_selected(matches!(self.current, Field::Token));
        self.url.set_selected(matches!(self.current, Field::Url));

        // Update the status message
        if !self.status.is_finished() && !self.status.is_loading() {
            if self.name.is_invalid() || self.token.is_invalid() || self.url.is_invalid() {
                self.status
                    .change(Status::Error, "Please fill in the fields");
            } else {
                self.status.change(Status::Ok, "Press ↵ to confirm");
            }
        }

        // Create areas for header, main, and footer
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header("Add a new controller", header_area, buffer);
        CreateWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl CreateWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to go back.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let area = WindowUtils::render_background(area, buffer);

        let [name_area, token_area, url_area, _, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1), // Empty space
            Constraint::Fill(1),
        ])
        .areas(area);

        self.name.render(name_area, buffer);
        self.token.render(token_area, buffer);
        self.url.render(url_area, buffer);

        self.status.render(status_area, buffer);
    }
}
