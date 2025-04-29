use std::sync::Arc;

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
        proto::manage::{
            group::{Constraints, Detail, Scaling},
            node::{self},
            server::{DiskRetention, Fallback, Resources, Spec},
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
    name: String,
    nodes: Vec<node::Short>,
    constraints: Constraints,
    scaling: Scaling,
    resources: Resources,

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

enum Field {
    Image,
    MaxPlayers,
    Settings,
    Environment,
    Retention,
    Fallback,
}

impl SpecificationWindow<'_> {
    pub fn new(
        connection: Arc<EstablishedConnection>,
        name: String,
        nodes: Vec<node::Short>,
        constraints: Constraints,
        scaling: Scaling,
        resources: Resources,
    ) -> Self {
        Self {
            name,
            nodes,
            constraints,
            scaling,
            resources,
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
                SimpleTextArea::not_empty_validation,
            ),
            environment: SimpleTextArea::new(
                (),
                "Environment (key=value,key1=value1)",
                "Please enter the environment that are passed to the plugin",
                SimpleTextArea::not_empty_validation,
            ),
            retention: SimpleTextArea::new(
                (),
                "Disk Retention (Permanent or not)",
                "true/false",
                SimpleTextArea::type_validation::<bool>,
            ),
            fallback: SimpleTextArea::new(
                (),
                "Fallback priority (Leave empty if this group is not a fallback group)",
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
                    {
                        // We use .unwrap because the values are validated by the text area
                        let max_players = self.max_players.get_first_line().parse::<u32>().unwrap();
                        let retention = self.retention.get_first_line().parse::<bool>().unwrap();
                        let fallback = self.fallback.get_first_line();
                        let fallback = if fallback.is_empty() {
                            None
                        } else {
                            Some(fallback.parse::<i32>().unwrap())
                        };

                        let spec = Spec {
                            img: self.image.get_first_line(),
                            max_players,
                            settings: vec![],
                            env: vec![],
                            retention: Some(if retention {
                                DiskRetention::Permanent as i32
                            } else {
                                DiskRetention::Temporary as i32
                            }),
                            fallback: fallback.map(|fallback| Fallback { prio: fallback }),
                        };

                        stack.pop(); // This is required to free the data stored in the struct
                        stack.push(CreateGroupTab::new(
                            self.connection.clone(),
                            Detail {
                                name: self.name.clone(),
                                nodes: self.nodes.iter().map(|item| item.name.clone()).collect(),
                                constraints: Some(self.constraints),
                                scaling: Some(self.scaling),
                                resources: Some(self.resources),
                                spec: Some(spec),
                            },
                        ));
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

        WindowUtils::render_tab_header("Group constraints", title_area, buffer);
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
