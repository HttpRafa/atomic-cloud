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
            node::{Capabilities, Detail},
            plugin,
        },
    },
    util::{
        area::SimpleTextArea,
        status::{Status, StatusDisplay},
    },
    window::{StackBatcher, Window, WindowUtils},
    State,
};

use super::CreateNodeTab;

pub struct CapabilitiesWindow<'a> {
    /* Data */
    name: String,
    url: String,
    plugin: plugin::Short,

    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,

    current: Field,

    memory: SimpleTextArea<'a, ()>,
    servers: SimpleTextArea<'a, ()>,
    child_node: SimpleTextArea<'a, ()>,
}

enum Field {
    Memory,
    Servers,
    ChildNode,
}

impl CapabilitiesWindow<'_> {
    pub fn new(
        connection: Arc<EstablishedConnection>,
        name: String,
        url: String,
        plugin: plugin::Short,
    ) -> Self {
        Self {
            name,
            url,
            plugin,
            connection,
            status: StatusDisplay::new(Status::Error, ""),
            current: Field::Memory,
            memory: SimpleTextArea::new_selected(
                (),
                "Memory in MiB(1024 MiB = 1 GB) [Not required]",
                "Please enter the amount of memory that the node can use",
                SimpleTextArea::optional_type_validation::<u32>,
            ),
            servers: SimpleTextArea::new(
                (),
                "Servers [Not required]",
                "Please enter the amount of servers this node can start",
                SimpleTextArea::optional_type_validation::<u32>,
            ),
            child_node: SimpleTextArea::new(
                (),
                "Child Node [Not required for some plugins like \"local\"]",
                "Please enter the name from the backend",
                SimpleTextArea::i_dont_care_validation,
            ),
        }
    }
}

#[async_trait]
impl Window for CapabilitiesWindow<'_> {
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
                    Field::Memory => {}
                    Field::Servers => self.current = Field::Memory,
                    Field::ChildNode => self.current = Field::Servers,
                },
                KeyCode::Down | KeyCode::Tab => match self.current {
                    Field::Memory => self.current = Field::Servers,
                    Field::Servers => self.current = Field::ChildNode,
                    Field::ChildNode => {}
                },
                KeyCode::Enter => {
                    if self.memory.is_valid()
                        && self.servers.is_valid()
                        && self.child_node.is_valid()
                    {
                        let memory = self.memory.get_first_line();
                        let servers = self.servers.get_first_line();
                        let child_node = self.child_node.get_first_line();
                        stack.pop(); // This is required to free the data stored in the struct
                                     // Use .clone() because we are lazy and .unwrap because the values are validated by the text area
                        stack.push(CreateNodeTab::new(
                            self.connection.clone(),
                            Detail {
                                name: self.name.clone(),
                                controller_address: self.url.clone(),
                                plugin: self.plugin.name.clone(),
                                capabilities: Some(Capabilities {
                                    memory: if memory.is_empty() {
                                        None
                                    } else {
                                        Some(memory.parse::<u32>().unwrap())
                                    },
                                    servers: if servers.is_empty() {
                                        None
                                    } else {
                                        Some(servers.parse::<u32>().unwrap())
                                    },
                                    child_node: if child_node.is_empty() {
                                        None
                                    } else {
                                        Some(child_node)
                                    },
                                }),
                            },
                        ));
                    }
                }
                _ => match self.current {
                    Field::Memory => self.memory.handle_event(event),
                    Field::Servers => self.servers.handle_event(event),
                    Field::ChildNode => self.child_node.handle_event(event),
                },
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut CapabilitiesWindow<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected fields
        self.memory
            .set_selected(matches!(self.current, Field::Memory));
        self.servers
            .set_selected(matches!(self.current, Field::Servers));
        self.child_node
            .set_selected(matches!(self.current, Field::ChildNode));

        // Update the status message
        if self.memory.is_valid() && self.servers.is_valid() && self.child_node.is_valid() {
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

        WindowUtils::render_tab_header("Node capabilities", title_area, buffer);
        CapabilitiesWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl CapabilitiesWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [memory_area, max_servers_area, child_node_area, _, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1), // Empty space
            Constraint::Fill(1),
        ])
        .areas(area);

        self.memory.render(memory_area, buffer);
        self.servers.render(max_servers_area, buffer);
        self.child_node.render(child_node_area, buffer);

        self.status.render(status_area, buffer);
    }
}
