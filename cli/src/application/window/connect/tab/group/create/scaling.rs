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
    network::{
        connection::EstablishedConnection,
        proto::manage::group::{Detail, Scaling},
    },
    util::{
        area::SimpleTextArea,
        status::{Status, StatusDisplay},
    },
    window::{StackBatcher, Window, WindowUtils},
};

use super::resources::ResourcesWindow;

pub struct ScalingWindow<'a> {
    /* Data */
    group: Option<Detail>,

    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,

    current: bool,

    start_threshold: SimpleTextArea<'a, ()>,
    stop_empty: SimpleTextArea<'a, ()>,
}

impl ScalingWindow<'_> {
    pub fn new(connection: Arc<EstablishedConnection>, group: Detail) -> Self {
        Self {
            group: Some(group),
            connection,
            status: StatusDisplay::new(Status::Error, ""),
            current: true,
            start_threshold: SimpleTextArea::new_selected(
                (),
                "Start Threshold (in 0-100%)",
                "At what percentage of max players should the auto scaler start a new server",
                SimpleTextArea::type_validation::<f32>,
            ),
            stop_empty: SimpleTextArea::new(
                (),
                "Stop empty servers",
                "true/false",
                SimpleTextArea::type_validation::<bool>,
            ),
        }
    }
}

#[async_trait]
impl Window for ScalingWindow<'_> {
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
                    if self.start_threshold.is_valid()
                        && self.stop_empty.is_valid()
                        && let Some(mut group) = self.group.take()
                    {
                        let start_threshold = self
                            .start_threshold
                            .get_first_line()
                            .parse::<f32>()
                            .expect("Should be validated by the text area")
                            / 100.0; // The controller expects a value from 0 to 1
                        let stop_empty = self
                            .stop_empty
                            .get_first_line()
                            .parse::<bool>()
                            .expect("Should be validated by the text area");

                        group.scaling = Some(Scaling {
                            enabled: true,
                            start_threshold,
                            stop_empty,
                        });

                        stack.pop(); // This is required to free the data stored in the struct
                        stack.push(ResourcesWindow::new(self.connection.clone(), group));
                    }
                }
                _ => {
                    if self.current {
                        self.start_threshold.handle_event(event);
                    } else {
                        self.stop_empty.handle_event(event);
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

impl Widget for &mut ScalingWindow<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected fields
        self.start_threshold.set_selected(self.current);
        self.stop_empty.set_selected(!self.current);

        // Update the status message
        if self.start_threshold.is_valid() && self.stop_empty.is_valid() {
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

        WindowUtils::render_tab_header("Group scaling", title_area, buffer);
        ScalingWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl ScalingWindow<'_> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to switch fields, ↵ to confirm, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let [start_threshold_area, stop_empty_area, _, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1), // Empty space
            Constraint::Fill(1),
        ])
        .areas(area);

        self.start_threshold.render(start_threshold_area, buffer);
        self.stop_empty.render(stop_empty_area, buffer);

        self.status.render(status_area, buffer);
    }
}
