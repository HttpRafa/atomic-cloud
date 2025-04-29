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
    network::connection::EstablishedConnection,
    util::{
        area::SimpleTextArea,
        status::{Status, StatusDisplay},
    },
    window::{
        connect::tab::util::{fetch::FetchWindow, multi_select::MultiSelectWindow},
        StackBatcher, Window, WindowUtils,
    },
    State,
};

use super::constraints::ConstraintsWindow;

pub struct BasicWindow<'a> {
    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,

    name: SimpleTextArea<'a, Vec<String>>,
}

impl BasicWindow<'_> {
    pub fn new(connection: Arc<EstablishedConnection>, groups: Vec<String>) -> Self {
        Self {
            connection,
            status: StatusDisplay::new(Status::Error, ""),
            name: SimpleTextArea::new_selected(
                groups,
                "Name",
                "Please enter the name of the group",
                SimpleTextArea::already_exists_validation,
            ),
        }
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
                KeyCode::Enter => {
                    if self.name.is_valid() {
                        let name = self.name.get_first_line();
                        stack.pop(); // This is required to free the data stored in the struct
                        stack.push(FetchWindow::new(
                            self.connection.get_nodes(),
                            self.connection.clone(),
                            move |nodes, connection, stack, _| {
                                stack.push(MultiSelectWindow::new(
                                    "Select the node/s the group can use to start servers on",
                                    nodes,
                                    move |nodes, stack, _| {
                                        stack.push(ConstraintsWindow::new(connection, name, nodes));
                                        Ok(())
                                    },
                                ));
                                Ok(())
                            },
                        ));
                    }
                }
                _ => self.name.handle_event(event),
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
        // Update the status message
        if self.name.is_valid() {
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

        WindowUtils::render_tab_header("Group basics", title_area, buffer);
        BasicWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl BasicWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↵ to confirm, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [name_area, _, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1), // Empty space
            Constraint::Fill(1),
        ])
        .areas(area);

        self.name.render(name_area, buffer);

        self.status.render(status_area, buffer);
    }
}
