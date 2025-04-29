use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::WHITE, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Widget},
};
use tonic::async_trait;
use url::Url;

use crate::application::{
    network::{connection::EstablishedConnection, proto::manage::plugin},
    util::{
        area::SimpleTextArea,
        status::{Status, StatusDisplay},
    },
    window::{
        connect::tab::util::{fetch::FetchWindow, select::SelectWindow},
        StackBatcher, Window,
    },
    State,
};

use super::capabilities::CapabilitiesWindow;

pub struct BasicWindow<'a> {
    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,

    current: bool,

    name: SimpleTextArea<'a, Vec<String>>,
    url: SimpleTextArea<'a, ()>,
}

impl BasicWindow<'_> {
    pub fn new(connection: Arc<EstablishedConnection>, nodes: Vec<String>) -> Self {
        Self { connection, status: StatusDisplay::new(Status::Error, ""), current: true, name: SimpleTextArea::new_selected(nodes, "Name", "Please enter the name of the node", SimpleTextArea::already_exists_validation), url: SimpleTextArea::new((), "URL of Controller (From the servers perspective)", "Please enter the url of the controller from the perspective of a started server on the node", SimpleTextArea::type_validation::<Url>) }
    }
}

#[async_trait]
impl Window for BasicWindow<'_> {
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
                KeyCode::Up => {
                    if !self.current {
                        self.current = true;
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    if self.current {
                        self.current = false;
                    }
                }
                KeyCode::Enter => {
                    if self.name.is_valid() {
                        let name = self.name.get_first_line();
                        let url = self.url.get_first_line();
                        stack.push(FetchWindow::new(
                            self.connection.get_plugins(),
                            self.connection.clone(),
                            move |plugins, connection, stack, _| {
                                stack.push(SelectWindow::new(
                                    "What plugin do you want to use of the node?",
                                    plugins,
                                    move |plugin, stack, _| {
                                        stack.push(CapabilitiesWindow::new(
                                            connection, name, url, plugin,
                                        ));
                                        Ok(())
                                    },
                                ));
                                Ok(())
                            },
                        ));
                    }
                }
                _ => {
                    if self.current {
                        self.name.handle_event(event);
                    } else {
                        self.url.handle_event(event);
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

impl Widget for &mut BasicWindow<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected fields
        self.name.set_selected(self.current);
        self.url.set_selected(!self.current);

        // Update the status message
        if self.name.is_valid() && self.url.is_valid() {
            self.status.change(Status::Ok, "Press ↵ to confirm");
        } else {
            self.status
                .change(Status::Error, "Please fill in the fields");
        }

        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        BasicWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl BasicWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [title_area, name_area, url_area, _, status_area] = Layout::vertical([
            Constraint::Length(2), // Title
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1), // Empty space
            Constraint::Fill(1),
        ])
        .areas(area);

        Paragraph::new("Node basics")
            .centered()
            .white()
            .bold()
            .render(title_area, buffer);

        let block = Block::default()
            .border_style(Style::default().not_bold().fg(WHITE))
            .borders(Borders::BOTTOM);
        block.render(title_area, buffer);

        self.name.render(name_area, buffer);
        self.url.render(url_area, buffer);

        self.status.render(status_area, buffer);
    }
}

impl Display for plugin::Short {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
