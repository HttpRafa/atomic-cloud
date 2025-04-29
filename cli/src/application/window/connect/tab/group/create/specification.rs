use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    str::FromStr,
    sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    network::{
        connection::EstablishedConnection,
        proto::{
            common::KeyValue,
            manage::{
                group::Detail,
                server::{DiskRetention, Fallback, Specification},
            },
        },
    },
    util::{
        area::SimpleTextArea,
        status::{Status, StatusDisplay},
    },
    window::{StackBatcher, Window, WindowUtils},
    State,
};

use super::CreateGroupTab;

pub struct SpecificationWindow<'a> {
    /* Data */
    group: Option<Detail>,

    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,

    current: Field,

    image: SimpleTextArea<'a, ()>,
    max_players: SimpleTextArea<'a, ()>,
    settings: SimpleTextArea<'a, ()>,
    environment: SimpleTextArea<'a, ()>,
    retention: SimpleTextArea<'a, ()>,
    fallback: SimpleTextArea<'a, ()>,
}

// Required to implement the FromStr trait
struct KeyValueList(Vec<KeyValue>);

enum Field {
    Image,
    MaxPlayers,
    Settings,
    Environment,
    Retention,
    Fallback,
}

impl SpecificationWindow<'_> {
    pub fn new(connection: Arc<EstablishedConnection>, group: Detail) -> Self {
        Self {
            group: Some(group),
            connection,
            status: StatusDisplay::new(Status::Error, ""),
            current: Field::Image,
            image: SimpleTextArea::new_selected(
                (),
                "Image",
                "Please enter the image to use for the server",
                SimpleTextArea::not_empty_validation,
            ),
            max_players: SimpleTextArea::new(
                (),
                "Max players",
                "Please enter the max amount of players per server",
                SimpleTextArea::type_validation::<u32>,
            ),
            settings: SimpleTextArea::new(
                (),
                "Settings (key=value,key1=value1)",
                "Please enter the setings that are passed to the plugin",
                SimpleTextArea::type_validation::<KeyValueList>,
            ),
            environment: SimpleTextArea::new(
                (),
                "Environment (key=value,key1=value1)",
                "Please enter the environment that are passed to the plugin",
                SimpleTextArea::type_validation::<KeyValueList>,
            ),
            retention: SimpleTextArea::new(
                (),
                "Disk Retention (Should the server's data be retained until the next start)",
                "yes/no or perm/temp",
                SimpleTextArea::type_validation::<DiskRetention>,
            ),
            fallback: SimpleTextArea::new(
                (),
                "Fallback priority (Not required if this group is not a fallback group)",
                "Please enter the priority for this group",
                SimpleTextArea::optional_type_validation::<i32>,
            ),
        }
    }
}

#[async_trait]
impl Window for SpecificationWindow<'_> {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // UI
        self.status.next();

        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        _state: &mut State,
        event: Event,
    ) -> Result<()> {
        if let Event::Key(event) = event {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => stack.close_tab(),
                KeyCode::Up => match self.current {
                    Field::Image => {}
                    Field::MaxPlayers => self.current = Field::Image,
                    Field::Settings => self.current = Field::MaxPlayers,
                    Field::Environment => self.current = Field::Settings,
                    Field::Retention => self.current = Field::Environment,
                    Field::Fallback => self.current = Field::Retention,
                },
                KeyCode::Down | KeyCode::Tab => match self.current {
                    Field::Image => self.current = Field::MaxPlayers,
                    Field::MaxPlayers => self.current = Field::Settings,
                    Field::Settings => self.current = Field::Environment,
                    Field::Environment => self.current = Field::Retention,
                    Field::Retention => self.current = Field::Fallback,
                    Field::Fallback => {}
                },
                KeyCode::Enter => {
                    if self.image.is_valid()
                        && self.max_players.is_valid()
                        && self.settings.is_valid()
                        && self.environment.is_valid()
                        && self.retention.is_valid()
                        && self.fallback.is_valid()
                        && let Some(mut group) = self.group.take()
                    {
                        let max_players = self
                            .max_players
                            .get_first_line()
                            .parse::<u32>()
                            .expect("Should be validated by the text area");
                        let settings = self
                            .settings
                            .get_first_line()
                            .parse::<KeyValueList>()
                            .expect("Should be validated by the text area");
                        let environment = self
                            .environment
                            .get_first_line()
                            .parse::<KeyValueList>()
                            .expect("Should be validated by the text area");
                        let retention = self
                            .retention
                            .get_first_line()
                            .parse::<DiskRetention>()
                            .expect("Should be validated by the text area");
                        let fallback = self.fallback.get_first_line();
                        let fallback = if fallback.is_empty() {
                            None
                        } else {
                            Some(
                                fallback
                                    .parse::<i32>()
                                    .expect("Should be validated by the text area"),
                            )
                        };

                        group.specification = Some(Specification {
                            image: self.image.get_first_line(),
                            max_players,
                            settings: settings.0,
                            environment: environment.0,
                            retention: Some(retention as i32),
                            fallback: fallback.map(|fallback| Fallback { priority: fallback }),
                        });

                        stack.pop(); // This is required to free the data stored in the struct
                        stack.push(CreateGroupTab::new(self.connection.clone(), group));
                    }
                }
                _ => match self.current {
                    Field::Image => self.image.handle_event(event),
                    Field::MaxPlayers => self.max_players.handle_event(event),
                    Field::Settings => self.settings.handle_event(event),
                    Field::Environment => self.environment.handle_event(event),
                    Field::Retention => self.retention.handle_event(event),
                    Field::Fallback => self.fallback.handle_event(event),
                },
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut SpecificationWindow<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected fields
        self.image
            .set_selected(matches!(self.current, Field::Image));
        self.max_players
            .set_selected(matches!(self.current, Field::MaxPlayers));
        self.settings
            .set_selected(matches!(self.current, Field::Settings));
        self.environment
            .set_selected(matches!(self.current, Field::Environment));
        self.retention
            .set_selected(matches!(self.current, Field::Retention));
        self.fallback
            .set_selected(matches!(self.current, Field::Fallback));

        // Update the status message
        if self.image.is_valid()
            && self.max_players.is_valid()
            && self.settings.is_valid()
            && self.environment.is_valid()
            && self.retention.is_valid()
            && self.fallback.is_valid()
        {
            self.status.change(Status::Ok, "Press ↵ to confirm");
        } else {
            self.status
                .change(Status::Error, "Please fill in the fields");
        }

        let [title_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_tab_header("Group specification", title_area, buffer);
        SpecificationWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl SpecificationWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [image_area, max_players_area, settings_area, environment_area, retention_area, fallback_area, _, status_area] =
            Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1), // Empty space
                Constraint::Fill(1),
            ])
            .areas(area);

        self.image.render(image_area, buffer);
        self.max_players.render(max_players_area, buffer);
        self.settings.render(settings_area, buffer);
        self.environment.render(environment_area, buffer);
        self.retention.render(retention_area, buffer);
        self.fallback.render(fallback_area, buffer);

        self.status.render(status_area, buffer);
    }
}

impl FromStr for DiskRetention {
    type Err = ParseRetentionError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "true" | "yes" | "perm" | "permanent" => Ok(Self::Permanent),
            "false" | "no" | "temp" | "temporary" => Ok(Self::Temporary),
            _ => Err(ParseRetentionError),
        }
    }
}

impl FromStr for KeyValueList {
    type Err = ParseKeyValueListError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut result = Vec::new();
        if string.is_empty() {
            return Ok(Self(result));
        }
        for pair in string.split(',') {
            let mut parts = pair.split('=');
            let key = parts
                .next()
                .ok_or_else(|| ParseKeyValueListError(format!("No key found in pair '{pair}'")))?;
            let value = parts.next().ok_or_else(|| {
                ParseKeyValueListError(format!("No value found for key '{key}' in pair '{pair}'"))
            })?;
            result.push(KeyValue {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
        Ok(Self(result))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRetentionError;

impl Display for ParseRetentionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        "Use: yes/no or perm/temp".fmt(formatter)
    }
}

impl Error for ParseRetentionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseKeyValueListError(String);

impl Display for ParseKeyValueListError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Error for ParseKeyValueListError {}
