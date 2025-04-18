use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, Frame};
use tonic::async_trait;

use crate::application::{network::known_host::manager::TrustRequest, State};

use super::{StackBatcher, Window};

pub struct TrustTlsWindow {
    request: TrustRequest,
}

impl TrustTlsWindow {
    pub fn new(request: TrustRequest) -> Self {
        Self { request }
    }
}

#[async_trait]
impl Window for TrustTlsWindow {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
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
            if event.code == KeyCode::Enter {}
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &mut TrustTlsWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {}
}
