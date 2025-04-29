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
            group::{Constraints, Scaling},
            node::{self},
            server::Resources,
        },
    },
    util::{
        area::SimpleTextArea,
        status::{Status, StatusDisplay},
    },
    window::{StackBatcher, Window, WindowUtils},
    State,
};

use super::specification::SpecificationWindow;

pub struct ResourcesWindow<'a> {
    /* Data */
    name: String,
    nodes: Vec<node::Short>,
    constraints: Constraints,
    scaling: Scaling,

    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,

    current: Field,

    memory: SimpleTextArea<'a, ()>,
    swap: SimpleTextArea<'a, ()>,
    cpu: SimpleTextArea<'a, ()>,
    io: SimpleTextArea<'a, ()>,
    disk: SimpleTextArea<'a, ()>,
    ports: SimpleTextArea<'a, ()>,
}

enum Field {
    Memory,
    Swap,
    Cpu,
    Io,
    Disk,
    Ports,
}

impl ResourcesWindow<'_> {
    pub fn new(
        connection: Arc<EstablishedConnection>,
        name: String,
        nodes: Vec<node::Short>,
        constraints: Constraints,
        scaling: Scaling,
    ) -> Self {
        Self {
            name,
            nodes,
            constraints,
            scaling,
            connection,
            status: StatusDisplay::new(Status::Error, ""),
            current: Field::Memory,
            memory: SimpleTextArea::new_selected(
                (),
                "MEMORY per server",
                "Please enter the max amount of MEMORY a server can use",
                SimpleTextArea::type_validation::<u32>,
            ),
            swap: SimpleTextArea::new(
                (),
                "SWAP per server",
                "Please enter the max amount of SWAP a server can use",
                SimpleTextArea::type_validation::<u32>,
            ),
            cpu: SimpleTextArea::new(
                (),
                "CPU Power (100 = 1 Core at 100%)",
                "Please enter the CPU power a server can use",
                SimpleTextArea::type_validation::<u32>,
            ),
            io: SimpleTextArea::new(
                (),
                "IO (IDK, default is 500)",
                "Please enter the IO a server can use",
                SimpleTextArea::type_validation::<u32>,
            ),
            disk: SimpleTextArea::new(
                (),
                "DISK",
                "Please enter the DISK a server can use",
                SimpleTextArea::type_validation::<u32>,
            ),
            ports: SimpleTextArea::new(
                (),
                "PORTS",
                "Please enter the amount of PORTS a server can use",
                SimpleTextArea::type_validation::<u32>,
            ),
        }
    }
}

#[async_trait]
impl Window for ResourcesWindow<'_> {
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
                    Field::Swap => self.current = Field::Memory,
                    Field::Cpu => self.current = Field::Swap,
                    Field::Io => self.current = Field::Cpu,
                    Field::Disk => self.current = Field::Io,
                    Field::Ports => self.current = Field::Disk,
                },
                KeyCode::Down | KeyCode::Tab => match self.current {
                    Field::Memory => self.current = Field::Swap,
                    Field::Swap => self.current = Field::Cpu,
                    Field::Cpu => self.current = Field::Io,
                    Field::Io => self.current = Field::Disk,
                    Field::Disk => self.current = Field::Ports,
                    Field::Ports => {}
                },
                KeyCode::Enter => {
                    if self.memory.is_valid()
                        && self.swap.is_valid()
                        && self.cpu.is_valid()
                        && self.io.is_valid()
                        && self.disk.is_valid()
                        && self.ports.is_valid()
                    {
                        // We use .unwrap because the values are validated by the text area
                        let memory = self.memory.get_first_line().parse::<u32>().unwrap();
                        let swap = self.swap.get_first_line().parse::<u32>().unwrap();
                        let cpu = self.cpu.get_first_line().parse::<u32>().unwrap();
                        let io = self.io.get_first_line().parse::<u32>().unwrap();
                        let disk = self.disk.get_first_line().parse::<u32>().unwrap();
                        let ports = self.ports.get_first_line().parse::<u32>().unwrap();

                        let resources = Resources {
                            memory,
                            swap,
                            cpu,
                            io,
                            disk,
                            ports,
                        };

                        stack.pop(); // This is required to free the data stored in the struct
                        stack.push(SpecificationWindow::new(
                            self.connection.clone(),
                            self.name.clone(),
                            self.nodes.clone(),
                            self.constraints,
                            self.scaling,
                            resources,
                        ));
                    }
                }
                _ => match self.current {
                    Field::Memory => self.memory.handle_event(event),
                    Field::Swap => self.swap.handle_event(event),
                    Field::Cpu => self.cpu.handle_event(event),
                    Field::Io => self.io.handle_event(event),
                    Field::Disk => self.disk.handle_event(event),
                    Field::Ports => self.ports.handle_event(event),
                },
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut ResourcesWindow<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected fields
        self.memory
            .set_selected(matches!(self.current, Field::Memory));
        self.swap.set_selected(matches!(self.current, Field::Swap));
        self.cpu.set_selected(matches!(self.current, Field::Cpu));
        self.io.set_selected(matches!(self.current, Field::Io));
        self.disk.set_selected(matches!(self.current, Field::Disk));
        self.ports
            .set_selected(matches!(self.current, Field::Ports));

        // Update the status message
        if self.memory.is_valid()
            && self.swap.is_valid()
            && self.cpu.is_valid()
            && self.io.is_valid()
            && self.disk.is_valid()
            && self.ports.is_valid()
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
        ResourcesWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl ResourcesWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [memory_area, swap_area, cpu_area, io_area, disk_area, ports_area, _, status_area] =
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

        self.memory.render(memory_area, buffer);
        self.swap.render(swap_area, buffer);
        self.cpu.render(cpu_area, buffer);
        self.io.render(io_area, buffer);
        self.disk.render(disk_area, buffer);
        self.ports.render(ports_area, buffer);

        self.status.render(status_area, buffer);
    }
}
