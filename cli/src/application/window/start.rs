use std::fmt::{Display, Formatter};

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{ListItem, Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    util::{list::ActionList, ERROR_SELECTED_COLOR, OK_SELECTED_COLOR, TEXT_FG_COLOR},
    State,
};

use super::{
    connect::ConnectWindow, create::CreateWindow, delete::DeleteWindow, StackBatcher, Window,
    WindowUtils,
};

pub struct StartWindow {
    list: ActionList<'static, Action>,
}

enum Action {
    Connect,
    Create,
    Delete,
}

impl Default for StartWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl StartWindow {
    pub fn new() -> Self {
        Self {
            list: ActionList::new(vec![Action::Connect, Action::Create, Action::Delete]),
        }
    }
}

#[async_trait]
impl Window for StartWindow {
    async fn init(&mut self, stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        if state.profiles.is_empty() {
            stack.push(CreateWindow::new(state));
        }
        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        state: &mut State,
        event: Event,
    ) -> Result<()> {
        if let Event::Key(event) = event {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => stack.pop(),
                KeyCode::Enter => {
                    if let Some(action) = self.list.selected() {
                        match *action {
                            Action::Connect => {
                                stack.push(ConnectWindow::new());
                            }
                            Action::Create => {
                                stack.push(CreateWindow::new(state));
                            }
                            Action::Delete => {
                                stack.push(DeleteWindow::new());
                            }
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

impl Widget for &mut StartWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header("Start", header_area, buffer);
        StartWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl StartWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to exit.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let area = WindowUtils::render_background(area, buffer);

        self.list.render(area, buffer);
    }
}

impl From<&Action> for ListItem<'_> {
    fn from(action: &Action) -> Self {
        match action {
            Action::Connect => ListItem::new(Line::styled(format!(" {action}"), TEXT_FG_COLOR)),
            Action::Create => ListItem::new(Line::styled(format!(" {action}"), OK_SELECTED_COLOR)),
            Action::Delete => {
                ListItem::new(Line::styled(format!(" {action}"), ERROR_SELECTED_COLOR))
            }
        }
    }
}

impl Display for Action {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Connect => write!(formatter, "Connect to existing controller"),
            Action::Create => write!(formatter, "Add new controller"),
            Action::Delete => write!(formatter, "Remove existing controller"),
        }
    }
}
