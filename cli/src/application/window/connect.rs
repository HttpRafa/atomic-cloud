use std::fmt::{Display, Formatter};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{ListItem, Paragraph, Widget},
};
use tab::TabsWindow;
use tonic::async_trait;

use crate::application::{
    network::connection::task::ConnectTask,
    profile::Profile,
    util::{
        center::CenterWarning,
        list::ActionList,
        status::{Status, StatusDisplay},
        TEXT_FG_COLOR,
    },
    State,
};

use super::{StackBatcher, Window, WindowUtils};

pub mod tab;

pub struct ConnectWindow {
    /* Handles */
    connect: Option<ConnectTask>,

    /* Window */
    status: StatusDisplay,
    list: Option<ActionList<'static, Profile>>,
}

impl Default for ConnectWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectWindow {
    pub fn new() -> Self {
        Self {
            connect: None,
            status: StatusDisplay::new(Status::Ok, "null"),
            list: None,
        }
    }
}

#[async_trait]
impl Window for ConnectWindow {
    async fn init(&mut self, _stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        self.list = Some(ActionList::new(
            state.profiles.profiles.values().cloned().collect(),
            true,
        ));
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Network connection
        if let Some(task) = &mut self.connect {
            match task.get_now().await {
                Ok(Some(Ok(connection))) => {
                    self.status
                        .change(Status::Successful, "Connected sucessfully!");
                    stack.push(TabsWindow::new(connection));
                }
                Err(error) | Ok(Some(Err(error))) => {
                    self.status
                        .change(Status::Fatal, format!("{}", error.root_cause()));
                }
                _ => {}
            }
        }

        // UI
        self.status.next();
        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        state: &mut State,
        event: Event,
    ) -> Result<()> {
        let Some(list) = self.list.as_mut() else {
            return Ok(());
        };

        if !self.status.is_loading()
            && let Event::Key(event) = event
        {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => stack.pop(),
                KeyCode::Enter => {
                    if !self.status.is_finished()
                        && let Some(profile) = list.selected()
                    {
                        self.status
                            .change_with_startpoint(Status::Loading, "Connecting to controller...");
                        self.connect =
                            Some(profile.establish_connection(state.known_hosts.clone()));
                    }
                }
                _ => {
                    if self.status.is_finished() && self.status.is_fatal() {
                        self.status.change(Status::Ok, "null");
                    }

                    list.handle_event(event);
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut ConnectWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header("Connect to existing controller", header_area, buffer);
        ConnectWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl ConnectWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to exit.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let area = WindowUtils::render_background(area, buffer);

        if let Some(list) = self.list.as_mut() {
            list.render(area, buffer);
            if list.is_empty() {
                CenterWarning::render(
                    "You dont have any existing controllers. Use Esc to exit.",
                    area,
                    buffer,
                );
            } else if !self.status.is_ok() {
                self.status.render_in_center(area, buffer);
            }
        }
    }
}

impl From<&Profile> for ListItem<'_> {
    fn from(profile: &Profile) -> Self {
        ListItem::new(Line::styled(format!(" {}", profile.name), TEXT_FG_COLOR))
    }
}

impl Display for Profile {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
