use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

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
        connection::EstablishedConnection,
        proto::{common::common_group, manage::group},
    },
    util::fancy_toml::FancyToml,
    window::{
        connect::tab::util::{fetch::FetchWindow, select::SelectWindow},
        StackBatcher, Window,
    },
    State,
};

pub struct GetGroupTab {
    /* Connection */
    group: group::Detail,

    /* Lines */
    lines: Vec<Line<'static>>,
}

impl GetGroupTab {
    /// Creates a new get group tab.
    /// This function will create a window stack to get the required information to display the group.
    pub fn new_stack(
        connection: Arc<EstablishedConnection>,
    ) -> FetchWindow<Vec<common_group::Short>> {
        FetchWindow::new(
            connection.get_groups(),
            connection,
            move |groups, connection: Arc<EstablishedConnection>, stack, _| {
                stack.push(SelectWindow::new(
                    "Select the group you want to view",
                    groups,
                    move |group, stack, _| {
                        stack.push(FetchWindow::new(
                            connection.get_group(&group.name),
                            connection,
                            move |group, _, stack, _| {
                                stack.push(GetGroupTab::new(group));
                                Ok(())
                            },
                        ));
                        Ok(())
                    },
                ));
                Ok(())
            },
        )
    }

    fn new(group: group::Detail) -> Self {
        Self {
            group,
            lines: vec![],
        }
    }
}

#[async_trait]
impl Window for GetGroupTab {
    async fn init(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Compute the lines
        if let Ok(toml) = toml::to_string_pretty(&self.group) {
            self.lines.extend(FancyToml::to_lines(&toml));
        }

        // Change the title
        stack.rename_tab(&self.group.name);

        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
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
            if event.code == KeyCode::Esc {
                stack.close_tab();
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut GetGroupTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        GetGroupTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl GetGroupTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines.clone()).render(area, buffer);
    }
}

impl Display for common_group::Short {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
