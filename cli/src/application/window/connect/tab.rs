use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::palette::tailwind::{self},
};
use start::StartTab;
use tonic::async_trait;

use crate::application::{State, network::connection::EstablishedConnection, tabs::Tabs};

use super::{StackBatcher, Window};

pub mod global;
pub mod group;
pub mod node;
pub mod server;
pub mod start;
pub mod user;
pub mod util;

pub struct TabsWindow {
    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Tabs */
    tabs: Tabs,
}

impl TabsWindow {
    pub fn new(connection: EstablishedConnection) -> Self {
        Self {
            tabs: Tabs::new(connection.get_name()),
            connection: Arc::new(connection),
        }
    }
}

#[async_trait]
impl Window for TabsWindow {
    async fn init(&mut self, _stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        self.tabs
            .add_tab(
                state,
                "Start".to_string(),
                tailwind::CYAN,
                Box::new(StartTab::new(self.connection.clone())),
            )
            .await?;

        Ok(())
    }

    async fn tick(&mut self, _stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        // Tick tab
        self.tabs.tick(state).await?;

        Ok(())
    }

    async fn handle_event(
        &mut self,
        stack: &mut StackBatcher,
        state: &mut State,
        event: Event,
    ) -> Result<()> {
        // Passthrough the event to the tab
        self.tabs.handle_event(state, event).await?;

        // Check if the user closed all tabs
        if self.tabs.is_empty() {
            stack.pops(2);
        }

        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        // Render tab
        self.tabs.render(area, buffer);
    }
}
