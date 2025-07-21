use std::{
    fmt::{Display, Formatter},
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
    State,
    network::{
        connection::EstablishedConnection,
        proto::manage::group::{Constraints, Detail, Scaling},
    },
    util::{
        area::SimpleTextArea,
        status::{Status, StatusDisplay},
    },
    window::{StackBatcher, Window, WindowUtils, connect::tab::util::select::SelectWindow},
};

use super::{resources::ResourcesWindow, scaling::ScalingWindow};

pub struct ConstraintsWindow<'a> {
    /* Data */
    group: Option<Detail>,

    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,

    current: Field,

    min_servers: SimpleTextArea<'a, ()>,
    max_servers: SimpleTextArea<'a, ()>,
    priority: SimpleTextArea<'a, ()>,
}

enum ScalingState {
    Use,
    DontUse,
}

enum Field {
    MinServers,
    MaxServers,
    Priority,
}

impl ConstraintsWindow<'_> {
    pub fn new(connection: Arc<EstablishedConnection>, group: Detail) -> Self {
        Self {
            group: Some(group),
            connection,
            status: StatusDisplay::new(Status::Error, ""),
            current: Field::MinServers,
            min_servers: SimpleTextArea::new_selected(
                (),
                "Min servers",
                "Please enter the amount of server that should be online all the time",
                SimpleTextArea::type_validation::<u32>,
            ),
            max_servers: SimpleTextArea::new(
                (),
                "Max servers",
                "Please enter the max amount of servers that can be started for this group",
                SimpleTextArea::type_validation::<u32>,
            ),
            priority: SimpleTextArea::new(
                (),
                "Priority (How important is this group compared to others?)",
                "Please enter the priority for this group",
                SimpleTextArea::type_validation::<i32>,
            ),
        }
    }
}

#[async_trait]
impl Window for ConstraintsWindow<'_> {
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
                    Field::MinServers => {}
                    Field::MaxServers => self.current = Field::MinServers,
                    Field::Priority => self.current = Field::MaxServers,
                },
                KeyCode::Down | KeyCode::Tab => match self.current {
                    Field::MinServers => self.current = Field::MaxServers,
                    Field::MaxServers => self.current = Field::Priority,
                    Field::Priority => {}
                },
                KeyCode::Enter => {
                    if self.min_servers.is_valid()
                        && self.max_servers.is_valid()
                        && self.priority.is_valid()
                        && let Some(mut group) = self.group.take()
                    {
                        let min_servers = self
                            .min_servers
                            .get_first_line()
                            .parse::<u32>()
                            .expect("Should be validated by the text area");
                        let max_servers = self
                            .max_servers
                            .get_first_line()
                            .parse::<u32>()
                            .expect("Should be validated by the text area");
                        let priority = self
                            .priority
                            .get_first_line()
                            .parse::<i32>()
                            .expect("Should be validated by the text area");

                        let connection = self.connection.clone();
                        group.constraints = Some(Constraints {
                            min_servers,
                            max_servers,
                            priority,
                        });

                        stack.pop(); // This is required to free the data stored in the struct
                        stack.push(SelectWindow::new(
                            "Do you want to use automatic server scaling with this group?",
                            vec![ScalingState::Use, ScalingState::DontUse],
                            move |state, stack, _| {
                                if matches!(state, ScalingState::Use) {
                                    stack.push(ScalingWindow::new(connection, group));
                                } else {
                                    // The user does not want to use automatic scaling
                                    group.scaling = Some(Scaling {
                                        enabled: false,
                                        start_threshold: 0.0,
                                        stop_empty: false,
                                    });

                                    stack.push(ResourcesWindow::new(connection, group));
                                }
                                Ok(())
                            },
                        ));
                    }
                }
                _ => match self.current {
                    Field::MinServers => self.min_servers.handle_event(event),
                    Field::MaxServers => self.max_servers.handle_event(event),
                    Field::Priority => self.priority.handle_event(event),
                },
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut ConstraintsWindow<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected fields
        self.min_servers
            .set_selected(matches!(self.current, Field::MinServers));
        self.max_servers
            .set_selected(matches!(self.current, Field::MaxServers));
        self.priority
            .set_selected(matches!(self.current, Field::Priority));

        // Update the status message
        if self.min_servers.is_valid() && self.max_servers.is_valid() && self.priority.is_valid() {
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
        ConstraintsWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl ConstraintsWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [
            memory_area,
            max_servers_area,
            child_node_area,
            _,
            status_area,
        ] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1), // Empty space
            Constraint::Fill(1),
        ])
        .areas(area);

        self.min_servers.render(memory_area, buffer);
        self.max_servers.render(max_servers_area, buffer);
        self.priority.render(child_node_area, buffer);

        self.status.render(status_area, buffer);
    }
}

impl Display for ScalingState {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalingState::Use => write!(formatter, "Yes (Use it)"),
            ScalingState::DontUse => write!(formatter, "No (Don't use it)"),
        }
    }
}
