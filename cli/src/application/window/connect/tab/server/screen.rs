use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use ansi_to_tui::IntoText;
use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use futures::FutureExt;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap,
    },
};
use tonic::{async_trait, Streaming};
use tui_textarea::TextArea;

use crate::application::{
    network::{
        connection::EstablishedConnection,
        proto::{
            common::common_server,
            manage::screen::{self, WriteReq},
        },
    },
    util::{
        status::{Status, StatusDisplay},
        TEXT_FG_COLOR,
    },
    window::{
        connect::tab::util::{fetch::FetchWindow, select::SelectWindow},
        StackBatcher, Window,
    },
    State,
};

pub struct ScreenTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,
    server: common_server::Short,
    stream: Streaming<screen::Lines>,

    /* State */
    status: StatusDisplay,

    /* Content */
    lines: Vec<Line<'static>>,
    scrollable_lines: usize,
    available_lines: u16,

    /* Scrollbar */
    scroll_state: ScrollbarState,
    scroll: usize,

    /* Input */
    command: TextArea<'static>,
}

impl ScreenTab {
    /// Creates a new screen tab.
    /// This function will create a window stack to get the required information to display the screen.
    pub fn new_stack(
        connection: Arc<EstablishedConnection>,
    ) -> FetchWindow<Vec<common_server::Short>> {
        FetchWindow::new(
            connection.get_servers(),
            connection,
            move |servers, connection: Arc<EstablishedConnection>, stack, _| {
                stack.push(SelectWindow::new(
                    "Select the server you want to open the screen for",
                    servers,
                    move |server, stack, _| {
                        stack.push(FetchWindow::new(
                            connection.subscribe_to_screen(&server.id),
                            connection,
                            move |screen, connection, stack, _| {
                                stack.push(ScreenTab::new(connection, server, screen));
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

    pub fn new(
        connection: Arc<EstablishedConnection>,
        server: common_server::Short,
        stream: Streaming<screen::Lines>,
    ) -> Self {
        let mut command = TextArea::default();
        command.set_cursor_line_style(Style::default());
        command.set_placeholder_text("Type the command you want to send");

        Self {
            connection,
            server,
            stream,
            status: StatusDisplay::new(Status::Ok, "All good"),
            lines: vec![],
            scrollable_lines: 0,
            available_lines: 0,
            scroll_state: ScrollbarState::default(),
            scroll: 0,
            command,
        }
    }
}

#[async_trait]
impl Window for ScreenTab {
    async fn init(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        stack.rename_tab(&self.server.name);
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Network stream
        match self.stream.message().now_or_never() {
            Some(Ok(Some(message))) => {
                // Handle new message
                for line in message.lines {
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(text) = line.into_text() {
                        for line in text {
                            self.lines.push(line);
                        }
                    }
                }
            }
            Some(Err(error)) => {
                // Handle error
                self.status.change(Status::Error, format!("{error}"));
                stack.rename_tab(format!("{} (error)", self.server.name));
            }
            Some(Ok(None)) => {
                // Handle end of stream
                self.status.change(
                    Status::NotPerfect,
                    "The network stream was closed by the other party. Use Esc to close the screen",
                );
                stack.rename_tab(format!("{} (closed)", self.server.name));
            }
            None => {} // No new message yet
        }

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
                KeyCode::Up => self.update_scroll(self.scroll.saturating_sub(1)),
                KeyCode::Down => self.update_scroll(self.scroll.saturating_add(1)),
                KeyCode::PageUp | KeyCode::Home => self.update_scroll(0),
                KeyCode::PageDown => self.update_scroll(self.scrollable_lines),
                KeyCode::Enter => {
                    // Write the command to the screen
                    let command = self
                        .command
                        .lines()
                        .first()
                        .expect("Should always return min one line");
                    if !command.is_empty() {
                        // Send the command to the server
                        let request = WriteReq {
                            id: self.server.id.clone(),
                            data: format!("{command}\n").into_bytes(),
                        };
                        // Fire and forget
                        // This will not block the UI
                        let _ = self.connection.write_to_screen(request);

                        self.command.delete_line_by_head();
                    }
                }
                _ => {
                    self.command.input(event);
                }
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
    fn update_scroll(&mut self, scroll: usize) {
        self.scroll = scroll.min(self.scrollable_lines);
        self.scroll_state = self.scroll_state.position(scroll);
    }

    fn update_lines(&mut self, total_lines: usize, viewport_height: u16) {
        let should_scroll_to_bottom = self.scroll == self.scrollable_lines;
        self.available_lines = viewport_height;
        self.scrollable_lines = total_lines.saturating_sub(viewport_height as usize);

        if should_scroll_to_bottom {
            // Automatically scroll to the bottom if previously at the bottom
            self.update_scroll(self.scrollable_lines);
        }
    }

    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↑↓ to scroll, Esc to close the screen.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [mut main_area, input_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        if !self.status.is_ok() {
            let [status_area, new_main_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(main_area);
            main_area = new_main_area;

            // If the status is not ok, we want to display the status
            self.status.render(status_area, buffer);
        }

        let [content_area, scrollbar_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(main_area);

        // Content area
        {
            #[allow(clippy::cast_possible_truncation)]
            let paragraph = Paragraph::new(self.lines.clone())
                .gray()
                .wrap(Wrap { trim: false })
                .scroll((self.scroll as u16, 0));

            // Calculate the height/lines of the content area
            self.update_lines(
                paragraph.line_count(content_area.width), // Might be removed in the future by ratatui
                content_area.height,
            );

            // Update the content length of the scrollbar
            self.scroll_state = self.scroll_state.content_length(self.scrollable_lines);

            // Render paragraph
            paragraph.render(content_area, buffer);

            StatefulWidget::render(
                Scrollbar::new(ScrollbarOrientation::VerticalRight).style(TEXT_FG_COLOR),
                scrollbar_area,
                buffer,
                &mut self.scroll_state,
            );
        }

        // Input area
        {
            let [symbol_area, main_area] =
                Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)]).areas(input_area);
            Paragraph::new("?")
                .left_aligned()
                .green()
                .bold()
                .render(symbol_area, buffer);
            self.command.render(main_area, buffer);
        }
    }
}

impl Display for common_server::Short {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
