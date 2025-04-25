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
    widgets::{ListItem, Paragraph, Widget},
};
use tonic::{async_trait, Streaming};

use crate::application::{
    network::{
        connection::EstablishedConnection,
        proto::manage::{screen, server},
    },
    util::TEXT_FG_COLOR,
    window::{
        controller::util::{fetch::FetchWindow, select::SelectWindow},
        StackBatcher, Window,
    },
    State,
};

pub struct ScreenTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,
    stream: Streaming<screen::Lines>,
}

impl ScreenTab {
    /// Creates a new screen tab.
    /// This function will create a window stack to get the required information to display the screen.
    pub fn collected(connection: Arc<EstablishedConnection>) -> FetchWindow<Vec<server::Short>> {
        FetchWindow::new(
            connection.get_servers(),
            connection,
            move |servers, connection: Arc<EstablishedConnection>, stack, _| {
                stack.push(SelectWindow::new(servers, move |server, stack, _| {
                    stack.push(FetchWindow::new(
                        connection.subscribe_to_screen(&server.id),
                        connection.clone(),
                        move |screen, connection, stack, _| {
                            stack.push(ScreenTab::new(connection, screen));
                            Ok(())
                        },
                    ));
                    Ok(())
                }));
                Ok(())
            },
        )
    }

    pub fn new(connection: Arc<EstablishedConnection>, stream: Streaming<screen::Lines>) -> Self {
        Self { connection, stream }
    }
}

#[async_trait]
impl Window for ScreenTab {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
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

impl Widget for &mut ScreenTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        ScreenTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl ScreenTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use Esc to close screen.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, _area: Rect, _buffer: &mut Buffer) {}
}

impl From<&server::Short> for ListItem<'_> {
    fn from(server: &server::Short) -> Self {
        ListItem::new(Line::styled(format!(" {server}"), TEXT_FG_COLOR))
    }
}

impl Display for server::Short {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
