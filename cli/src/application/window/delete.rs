use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::{ListItem, Paragraph, Widget},
    Frame,
};
use tonic::async_trait;

use crate::application::{
    profile::Profile,
    util::{list::ActionList, TEXT_FG_COLOR},
    State,
};

use super::{StackBatcher, Window, WindowUtils};

#[derive(Default)]
pub struct DeleteWindow {
    list: Option<ActionList<Profile>>,
}

#[async_trait]
impl Window for DeleteWindow {
    async fn init(&mut self, _stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        self.list = Some(ActionList::new(
            state.profiles.profiles.values().cloned().collect(),
        ));
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
        let Some(list) = self.list.as_mut() else {
            return Ok(());
        };

        if let Event::Key(event) = event {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            match event.code {
                KeyCode::Esc => stack.pop(),
                KeyCode::Enter => {}
                KeyCode::Down => list.next(),
                KeyCode::Up => list.previous(),
                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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

        if let Some(list) = self.list.as_mut() {
            list.render(main_area, buffer);
        }
    }
}

impl DeleteWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to exit.")
            .centered()
            .render(area, buffer);
    }
}

impl From<&Profile> for ListItem<'_> {
    fn from(profile: &Profile) -> Self {
        ListItem::new(Line::styled(format!(" {}", profile.name), TEXT_FG_COLOR))
    }
}
