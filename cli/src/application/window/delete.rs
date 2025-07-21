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
    State,
    profile::Profile,
    util::{ERROR_SELECTED_COLOR, TEXT_FG_COLOR, center::CenterWarning, list::ActionList},
};

use super::{StackBatcher, Window, WindowUtils};

struct ListProfile {
    inner: Profile,
    delete: bool,
}

// TODO: Rewrite this to use the new SelectWindow implementation instead of ActionList directly
// May require some changes to the SelectWindow implementation to support async callbacks
pub struct DeleteWindow {
    list: Option<ActionList<'static, ListProfile>>,
}

impl Default for DeleteWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl DeleteWindow {
    pub fn new() -> Self {
        Self { list: None }
    }
}

#[async_trait]
impl Window for DeleteWindow {
    async fn init(&mut self, _stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        self.list = Some(ActionList::new(
            state
                .profiles
                .profiles
                .values()
                .cloned()
                .map(|profile| ListProfile {
                    inner: profile,
                    delete: false,
                })
                .collect(),
            true,
        ));
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
        let Some(list) = self.list.as_mut() else {
            return Ok(());
        };

        if let Event::Key(event) = event {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => stack.pop(),
                KeyCode::Enter => {
                    if let Some(profile) = list.selected_mut() {
                        if profile.delete {
                            // Delete profile
                            state.profiles.delete_profile(&profile.inner).await?;
                            stack.pop();
                        } else {
                            // Request confirmation from user
                            profile.delete = true;
                        }
                    }
                }
                _ => list.handle_event(event),
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut DeleteWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header("Remove existing controller", header_area, buffer);
        DeleteWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl DeleteWindow {
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
            }
        }
    }
}

impl From<&ListProfile> for ListItem<'_> {
    fn from(profile: &ListProfile) -> Self {
        let (text, color) = if profile.delete {
            (
                format!(" {} (Confirm)", profile.inner.name),
                ERROR_SELECTED_COLOR,
            )
        } else {
            (format!(" {}", profile.inner.name), TEXT_FG_COLOR)
        };
        ListItem::new(Line::styled(text, color))
    }
}

impl Display for ListProfile {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.inner)
    }
}
