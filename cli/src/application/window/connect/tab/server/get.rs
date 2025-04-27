use std::sync::Arc;

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
        proto::manage::server::{self},
    },
    util::fancy_toml::FancyToml,
    window::{
        connect::tab::util::{fetch::FetchWindow, select::SelectWindow},
        StackBatcher, Window,
    },
    State,
};

pub struct GetServerTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,
    server: server::Detail,

    /* Lines */
    lines: Vec<Line<'static>>,
}

impl GetServerTab {
    /// Creates a new get server tab.
    /// This function will create a window stack to get the required information to display the server.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> FetchWindow<Vec<server::Short>> {
        FetchWindow::new(
            connection.get_servers(),
            connection,
            move |servers, connection: Arc<EstablishedConnection>, stack, _| {
                stack.push(SelectWindow::new(servers, move |server, stack, _| {
                    stack.push(FetchWindow::new(
                        connection.get_server(&server.id),
                        connection.clone(),
                        move |server, connection, stack, _| {
                            stack.push(GetServerTab::new(connection.clone(), server));
                            Ok(())
                        },
                    ));
                    Ok(())
                }));
                Ok(())
            },
        )
    }

    pub fn new(connection: Arc<EstablishedConnection>, server: server::Detail) -> Self {
        Self {
            connection,
            server,
            lines: vec![],
        }
    }
}

#[async_trait]
impl Window for GetServerTab {
    async fn init(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Compute the lines
        if let Ok(toml) = toml::to_string_pretty(&self.server) {
            self.lines.extend(FancyToml::to_lines(&toml));
        }

        // Change the title
        stack.rename_tab(&self.server.name);

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

impl Widget for &mut GetServerTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        GetServerTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl GetServerTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines.clone()).render(area, buffer);
    }
}
