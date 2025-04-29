use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::palette::tailwind::{BLUE, GREEN, RED, SLATE},
    text::Line,
    widgets::{ListItem, Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    network::connection::EstablishedConnection,
    util::{
        list::ActionList, ERROR_SELECTED_COLOR, INFO_SELECTED_COLOR, OK_SELECTED_COLOR,
        TEXT_FG_COLOR,
    },
    window::{StackBatcher, Window},
    State,
};

use super::{
    global::{delete::DeleteTab, set_active::SetActiveTab, stop::StopTab, version::VersionTab},
    group::{create::CreateGroupTab, get::GetGroupTab},
    node::{create::CreateNodeTab, get::GetNodeTab},
    server::{get::GetServerTab, screen::ScreenTab},
    user::transfer::TransferUserTab,
};

pub struct StartTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    list: ActionList<'static, Action>,
}

enum Action {
    // Resource operations
    SetResource,
    DeleteResource,

    // Node operations
    CreateNode,
    GetNode,

    // Group operations
    CreateGroup,
    GetGroup,

    // Server operations
    GetServer,

    // Screen operations
    OpenScreen,

    // Transfer operations
    TransferUsers,

    // General
    RequestStop,
    GetVersions,
}

impl StartTab {
    pub fn new(connection: Arc<EstablishedConnection>) -> Self {
        Self {
            connection,
            list: ActionList::new(
                vec![
                    Action::CreateNode,
                    Action::CreateGroup,
                    Action::SetResource,
                    Action::OpenScreen,
                    Action::TransferUsers,
                    Action::GetNode,
                    Action::GetGroup,
                    Action::GetServer,
                    Action::GetVersions,
                    Action::DeleteResource,
                    Action::RequestStop,
                ],
                false,
            ),
        }
    }
}

#[async_trait]
impl Window for StartTab {
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
            match event.code {
                KeyCode::Esc => stack.close_tab(),
                KeyCode::Enter => {
                    if let Some(action) = self.list.selected() {
                        match action {
                            Action::SetResource => stack.add_tab(
                                "Active",
                                GREEN,
                                SetActiveTab::new_stack(self.connection.clone()),
                            ),
                            Action::DeleteResource => stack.add_tab(
                                "Delete",
                                RED,
                                DeleteTab::new_stack(self.connection.clone()),
                            ),

                            Action::CreateNode => stack.add_tab(
                                "Create",
                                GREEN,
                                CreateNodeTab::new_stack(self.connection.clone()),
                            ),
                            Action::GetNode => stack.add_tab(
                                "Node",
                                SLATE,
                                GetNodeTab::new_stack(self.connection.clone()),
                            ),

                            Action::CreateGroup => stack.add_tab(
                                "Create",
                                GREEN,
                                CreateGroupTab::new(self.connection.clone()),
                            ),
                            Action::GetGroup => stack.add_tab(
                                "Group",
                                SLATE,
                                GetGroupTab::new_stack(self.connection.clone()),
                            ),

                            Action::GetServer => stack.add_tab(
                                "Server",
                                SLATE,
                                GetServerTab::new_stack(self.connection.clone()),
                            ),

                            Action::OpenScreen => stack.add_tab(
                                "Screen",
                                BLUE,
                                ScreenTab::new_stack(self.connection.clone()),
                            ),

                            Action::TransferUsers => stack.add_tab(
                                "Transfer",
                                BLUE,
                                TransferUserTab::new_stack(self.connection.clone()),
                            ),

                            Action::RequestStop => {
                                stack.add_tab(
                                    "Stop",
                                    RED,
                                    StopTab::new_stack(self.connection.clone()),
                                );
                            }
                            Action::GetVersions => stack.add_tab(
                                "Versions",
                                SLATE,
                                VersionTab::new_stack(self.connection.clone()),
                            ),
                        }
                    }
                }
                _ => self.list.handle_event(event),
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut StartTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        StartTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl StartTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        self.list.render(area, buffer);
    }
}

impl From<&Action> for ListItem<'_> {
    fn from(action: &Action) -> Self {
        match action {
            Action::CreateGroup | Action::CreateNode | Action::SetResource => {
                ListItem::new(Line::styled(format!(" {action}"), OK_SELECTED_COLOR))
            }
            Action::DeleteResource | Action::RequestStop => {
                ListItem::new(Line::styled(format!(" {action}"), ERROR_SELECTED_COLOR))
            }
            Action::OpenScreen | Action::TransferUsers => {
                ListItem::new(Line::styled(format!(" {action}"), INFO_SELECTED_COLOR))
            }
            _ => ListItem::new(Line::styled(format!(" {action}"), TEXT_FG_COLOR)),
        }
    }
}

impl Display for Action {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::SetResource => write!(formatter, "Activate nodes or groups"),
            Action::DeleteResource => write!(formatter, "Delete nodes, groups or stop servers"),

            Action::CreateNode => write!(formatter, "Create Node"),
            Action::GetNode => write!(formatter, "Get information about a certain Node"),

            Action::CreateGroup => write!(formatter, "Create Group"),
            Action::GetGroup => write!(formatter, "Get information about a certain Group"),

            Action::GetServer => write!(formatter, "Get information about a certain Server"),

            Action::OpenScreen => write!(formatter, "Open the screen of a server"),

            Action::TransferUsers => write!(formatter, "Transfer a users to a different Server"),

            Action::RequestStop => write!(formatter, "Request stop of Controller"),
            Action::GetVersions => write!(formatter, "Get versions"),
        }
    }
}
