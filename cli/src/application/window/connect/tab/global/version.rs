use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, Widget},
};
use serde::Serialize;
use tonic::async_trait;

use crate::{
    application::{
        network::connection::EstablishedConnection,
        util::fancy_toml::FancyToml,
        window::{connect::tab::util::fetch::FetchWindow, StackBatcher, Window},
        State,
    },
    VERSION,
};

pub struct VersionTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,
    values: Values,

    /* Lines */
    lines: Vec<Line<'static>>,
}

impl VersionTab {
    /// Creates a new version tab.
    /// This function will create a window stack to get the required information to display the versions.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> FetchWindow<String> {
        FetchWindow::new(
            connection.get_ctrl_ver(),
            connection,
            move |version, connection: Arc<EstablishedConnection>, stack, _| {
                let protocol = connection.get_protocol();
                stack.push(VersionTab::new(
                    connection,
                    Values {
                        client: Client {
                            protocol: VERSION.protocol,
                            version: format!("{VERSION}"),
                        },
                        server: Server { protocol, version },
                    },
                ));
                Ok(())
            },
        )
    }

    pub fn new(connection: Arc<EstablishedConnection>, values: Values) -> Self {
        Self {
            connection,
            values,
            lines: vec![],
        }
    }
}

#[async_trait]
impl Window for VersionTab {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Compute the lines
        if let Ok(toml) = toml::to_string_pretty(&self.values) {
            self.lines.extend(FancyToml::to_lines(&toml));
        }

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

impl Widget for &mut VersionTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        VersionTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl VersionTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines.clone()).render(area, buffer);
    }
}

#[derive(Serialize)]
struct Values {
    client: Client,
    server: Server,
}

#[derive(Serialize)]
struct Client {
    protocol: u32,
    version: String,
}

#[derive(Serialize)]
struct Server {
    protocol: u32,
    version: String,
}
