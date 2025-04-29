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
        proto::manage::group,
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
mod constraints;
mod resources;
mod scaling;
mod specification;

// SEE: https://github.com/HttpRafa/atomic-cloud/blob/main/protocol/grpc/manage/group.proto

pub struct CreateGroupTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,
    group: group::Detail,

    /* Lines */
    lines: Vec<Line<'static>>,

    /* Window */
    request: Option<(EmptyTask, StatusDisplay)>,
}

impl CreateGroupTab {
    /// Creates a new create group tab.
    /// This function will create a window stack to get the required information to create a group.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> FetchWindow<Vec<group::Short>> {
        FetchWindow::new(
            connection.get_groups(),
            connection,
            move |groups, connection, stack, _| {
                stack.push(BasicWindow::new(
                    connection,
                    groups.into_iter().map(|group| group.name).collect(),
                ));
                Ok(())
            },
        )
    }

    pub fn new(connection: Arc<EstablishedConnection>, group: group::Detail) -> Self {
        Self {
            connection,
            group,
            lines: vec![],
            request: None,
        }
    }
}

#[async_trait]
impl Window for CreateGroupTab {
    async fn init(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Compute the lines
        if let Ok(toml) = toml::to_string_pretty(&self.group) {
            self.lines.extend(FancyToml::to_lines(&toml));
        }

        // Change the title
        stack.rename_tab(&self.group.name);

        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        if let Some((request, status)) = &mut self.request {
            // Network connection
            match request.get_now().await {
                Ok(Some(Ok(()))) => {
                    status.change_with_startpoint(
                        Status::Successful,
                        "Sucessfully created group on controller",
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
                        // Create group on controller
                        self.request = Some((
                            self.connection.create_group(self.group.clone()),
                            StatusDisplay::new_with_startpoint(
                                Status::Loading,
                                "Creating group on controller",
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

impl Widget for &mut CreateGroupTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        CreateGroupTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl CreateGroupTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close tab.")
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
