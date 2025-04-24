use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{ListItem, Widget},
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
    GetNodes,

    // Group operations
    CreateGroup,
    GetGroup,
    GetGroups,

    // Server operations
    GetServer,
    GetServers,

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
            list: ActionList::new(vec![
                Action::CreateNode,
                Action::CreateGroup,
                Action::SetResource,
                Action::OpenScreen,
                Action::TransferUsers,
                Action::GetNode,
                Action::GetNodes,
                Action::GetGroup,
                Action::GetGroups,
                Action::GetServer,
                Action::GetServers,
                Action::GetVersions,
                Action::RequestStop,
                Action::DeleteResource,
            ]),
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
                KeyCode::Enter => if let Some(_action) = self.list.selected() {},
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
        self.render_body(area, buffer);
    }
}

impl StartTab {
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
            Action::SetResource => write!(formatter, "Activate node or group"),
            Action::DeleteResource => write!(formatter, "Delete node, group or stop server"),

            Action::CreateNode => write!(formatter, "Create Node"),
            Action::GetNode => write!(formatter, "Get information about a certain Node"),
            Action::GetNodes => write!(formatter, "Get all Nodes"),

            Action::CreateGroup => write!(formatter, "Create Group"),
            Action::GetGroup => write!(formatter, "Get information about a certain Group"),
            Action::GetGroups => write!(formatter, "Get all Groups"),

            Action::GetServer => write!(formatter, "Get information about a certain Server"),
            Action::GetServers => write!(formatter, "Get all Servers"),

            Action::OpenScreen => write!(formatter, "Open the screen of a server"),

            Action::TransferUsers => write!(formatter, "Transfer a users to a different Server"),

            Action::RequestStop => write!(formatter, "Request stop of Controller"),
            Action::GetVersions => write!(formatter, "Get versions"),
        }
    }
}
