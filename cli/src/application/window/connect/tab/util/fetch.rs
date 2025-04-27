use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    network::connection::{task::NetworkTask, EstablishedConnection},
    util::status::{Status, StatusDisplay},
    window::{StackBatcher, Window},
    State,
};

type Callback<T> = Box<
    dyn Fn(T, Arc<EstablishedConnection>, &mut StackBatcher, &mut State) -> Result<()>
        + Send
        + 'static,
>;

pub struct FetchWindow<T> {
    /* Callback */
    callback: Callback<T>,

    /* Network */
    request: NetworkTask<Result<T>>,

    /* Connection */
    connection: Arc<EstablishedConnection>,

    /* Window */
    status: StatusDisplay,
}

impl<T> FetchWindow<T> {
    pub fn new<F>(
        request: NetworkTask<Result<T>>,
        connection: Arc<EstablishedConnection>,
        callback: F,
    ) -> Self
    where
        F: Fn(T, Arc<EstablishedConnection>, &mut StackBatcher, &mut State) -> Result<()>
            + Send
            + 'static,
    {
        Self {
            request,
            connection,
            callback: Box::new(callback),
            status: StatusDisplay::new(Status::Loading, "Retreiving required information..."),
        }
    }
}

#[async_trait]
impl<T: Send> Window for FetchWindow<T> {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, state: &mut State) -> Result<()> {
        // Network connection
        match self.request.get_now().await {
            Ok(Some(Ok(connection))) => {
                self.status.change(
                    Status::Successful,
                    "Sucessfully retrieved the required information", // Not really visible
                );
                stack.pop();
                (self.callback)(connection, self.connection.clone(), stack, state)?;
            }
            Err(error) | Ok(Some(Err(error))) => {
                self.status
                    .change(Status::Fatal, format!("{}", error.root_cause()));
            }
            _ => {}
        }

        // UI
        self.status.next();
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
            if event.code == KeyCode::Esc {
                self.request.abort();
                stack.pop();
                stack.close_tab();
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl<T> Widget for &mut FetchWindow<T> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        FetchWindow::<T>::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl<T> FetchWindow<T> {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use Esc to cancel.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        self.status.render_in_center(area, buffer);
    }
}
