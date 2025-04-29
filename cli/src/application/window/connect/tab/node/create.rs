use std::sync::Arc;

use basic::BasicWindow;
use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    network::{
        connection::{task::EmptyTask, EstablishedConnection},
        proto::manage::node,
    },
    util::{
        fancy_toml::FancyToml,
        status::{Status, StatusDisplay},
    },
    window::{
        connect::tab::{global::delete::AUTO_CLOSE_AFTER, util::fetch::FetchWindow},
        StackBatcher, Window,
    },
    State,
};

mod basic;
mod capabilities;

// SEE: https://github.com/HttpRafa/atomic-cloud/blob/main/protocol/grpc/manage/node.proto

pub struct CreateNodeTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,
    node: node::Detail,

    /* Lines */
    lines: Vec<Line<'static>>,

    /* Window */
    request: Option<(EmptyTask, StatusDisplay)>,
}

impl CreateNodeTab {
    /// Creates a new create node tab.
    /// This function will create a window stack to get the required information to create a node.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> FetchWindow<Vec<node::Short>> {
        FetchWindow::new(
            connection.get_nodes(),
            connection,
            move |nodes, connection, stack, _| {
                stack.push(BasicWindow::new(
                    connection,
                    nodes.into_iter().map(|node| node.name).collect(),
                ));
                Ok(())
            },
        )
    }

    pub fn new(connection: Arc<EstablishedConnection>, node: node::Detail) -> Self {
        Self {
            connection,
            node,
            lines: vec![],
            request: None,
        }
    }
}

#[async_trait]
impl Window for CreateNodeTab {
    async fn init(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Compute the lines
        if let Ok(toml) = toml::to_string_pretty(&self.node) {
            self.lines.extend(FancyToml::to_lines(&toml));
        }

        // Change the title
        stack.rename_tab(&self.node.name);

        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        if let Some((request, status)) = &mut self.request {
            // Network connection
            match request.get_now().await {
                Ok(Some(Ok(()))) => {
                    status.change_with_startpoint(
                        Status::Successful,
                        "Sucessfully created node on controller",
                    );
                }
                Err(error) | Ok(Some(Err(error))) => {
                    status.change(Status::Fatal, format!("{}", error.root_cause()));
                }
                _ => {}
            }

            // UI
            status.next();
            if status.elapsed() > AUTO_CLOSE_AFTER {
                stack.close_tab();
            }
        }
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
                KeyCode::Esc => {
                    if let Some((request, _)) = &mut self.request {
                        request.abort();
                    }
                    stack.close_tab();
                }
                KeyCode::Enter => {
                    if self.request.is_none() {
                        // Create node on controller
                        self.request = Some((
                            self.connection.create_node(self.node.clone()),
                            StatusDisplay::new_with_startpoint(
                                Status::Loading,
                                "Creating node on controller",
                            ),
                        ));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut CreateNodeTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        CreateNodeTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl CreateNodeTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↵ to create, Esc to cancel.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines.clone()).render(area, buffer);

        if let Some((_, status)) = &mut self.request {
            status.render_in_center(area, buffer);
        }
    }
}
