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
    util::{list::ActionList, TEXT_FG_COLOR},
    State,
};

use super::{create::CreateWindow, StackBatcher, Window, WindowUtils};

#[derive(Default)]
pub struct StartWindow {
    list: Option<ActionList<Action>>,
}

enum Action {
    Load,
    Create,
    Delete,
}

#[async_trait]
impl Window for StartWindow {
    async fn init(&mut self, stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        if state.profiles.is_empty() {
            stack.push(Box::new(CreateWindow::default()));
        }
        self.list = Some(ActionList::new(vec![
            Action::Load,
            Action::Create,
            Action::Delete,
        ]));
        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn handle_event(&mut self, stack: &mut StackBatcher, event: Event) -> Result<()> {
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
                    if let Some(action) = list.selected() {
                        if matches!(action, Action::Create) {
                            stack.push(Box::new(CreateWindow::default()));
                        }
                    }
                }
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

        if let Some(list) = self.list.as_mut() {
            list.render(main_area, buffer);
        }
    }
}

impl StartWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to exit.")
            .centered()
            .render(area, buffer);
    }
}

impl From<&Action> for ListItem<'_> {
    fn from(action: &Action) -> Self {
        let line = match action {
            Action::Load => Line::styled(" Connect to existing controller", TEXT_FG_COLOR),
            Action::Create => Line::styled(" Add new controller", TEXT_FG_COLOR),
            Action::Delete => Line::styled(" Remove existing controller", TEXT_FG_COLOR),
        };
        ListItem::new(line)
    }
}
