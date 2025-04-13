use color_eyre::eyre::Result;
use crossterm::event::Event;
use ratatui::{style::{Style, Stylize}, widgets::Block, Frame};
use tonic::async_trait;

use crate::application::Shared;

use super::{StackAction, Window};

pub struct StartWindow;

#[async_trait]
impl Window for StartWindow {
    async fn init(&mut self, _shared: &Shared) -> Result<StackAction> {
        Ok(StackAction::Nothing)
    }

    async fn tick(&mut self, _shared: &Shared) -> Result<StackAction> {
        Ok(StackAction::Nothing)
    }

    async fn handle_event(&mut self, _event: Event) -> Result<StackAction> {
        Ok(StackAction::Nothing)
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(Block::new().title("Atomic Cloud CLI").style(Style::new().blue().bold()), frame.area());
    }
}
