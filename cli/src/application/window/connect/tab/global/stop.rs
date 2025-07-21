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
    State,
    network::connection::{EstablishedConnection, task::EmptyTask},
    util::status::{Status, StatusDisplay},
    window::{StackBatcher, Window, connect::tab::util::confirm::ConfirmWindow},
};

use super::delete::AUTO_CLOSE_AFTER;

pub struct StopTab {
    /* Network */
    request: EmptyTask,

    /* Window */
    status: StatusDisplay,
}

impl StopTab {
    /// Creates a new stop tab.
    /// This function will create a window stack to ask the user if he wants to stop the controller.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> ConfirmWindow<'static> {
        ConfirmWindow::new(
            "Stop controller",
            "Are you sure you want to stop the controller?\nThis will stop all running servers.",
            ("Yes", "Stop"),
            ("No", "Cancel"),
            move |result, stack, _| {
                if result {
                    stack.push(StopTab::new(&connection));
                } else {
                    stack.close_tab();
                }
                Ok(())
            },
        )
    }

    pub fn new(connection: &Arc<EstablishedConnection>) -> Self {
        Self {
            request: connection.request_stop(),
            status: StatusDisplay::new_with_startpoint(Status::Loading, "Stopping controller..."),
        }
    }
}

#[async_trait]
impl Window for StopTab {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Network connection
        match self.request.get_now().await {
            Ok(Some(Ok(()))) => {
                self.status.change_with_startpoint(
                    Status::Successful,
                    "Sucessfully stopped the controller.",
                );
            }
            Err(error) | Ok(Some(Err(error))) => {
                self.status
                    .change(Status::Fatal, format!("{}", error.root_cause()));
            }
            _ => {}
        }

        // UI
        self.status.next();
        if self.status.is_successful() && self.status.elapsed() > AUTO_CLOSE_AFTER {
            stack.close_tab();
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
            if event.code == KeyCode::Esc {
                self.request.abort();
                stack.close_tab();
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut StopTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        StopTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl StopTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        self.status.render_in_center(area, buffer);
    }
}
