use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use tonic::async_trait;

use crate::application::{
    network::known_host::requests::TrustRequest,
    util::{button::SimpleButton, ERROR_COLOR, ERROR_SELECTED_COLOR, OK_COLOR, OK_SELECTED_COLOR},
    State,
};

use super::{StackBatcher, Window, WindowUtils};

pub struct TrustTlsWindow {
    request: TrustRequest,

    current: Button,

    yes_button: SimpleButton<'static>,
    no_button: SimpleButton<'static>,
}

enum Button {
    Yes,
    No,
}

impl TrustTlsWindow {
    pub fn new(request: TrustRequest) -> Self {
        Self {
            request,
            current: Button::No,
            yes_button: SimpleButton::new(
                "Yes",
                "Trust this certificate",
                (OK_SELECTED_COLOR, OK_COLOR),
            ),
            no_button: SimpleButton::new(
                "No",
                "Do not trust this certificate",
                (ERROR_SELECTED_COLOR, ERROR_COLOR),
            ),
        }
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
            match event.code {
                KeyCode::Right | KeyCode::Tab => match self.current {
                    Button::Yes => self.current = Button::No,
                    Button::No => {}
                },
                KeyCode::Left => match self.current {
                    Button::No => self.current = Button::Yes,
                    Button::Yes => {}
                },
                KeyCode::Enter => match self.current {
                    Button::Yes => {
                        state.known_hosts.set_trust(true, &mut self.request).await?;
                        stack.pop();
                    }
                    Button::No => {
                        state
                            .known_hosts
                            .set_trust(false, &mut self.request)
                            .await?;
                        stack.pop();
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut TrustTlsWindow {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        // Update the selected field
        self.yes_button
            .set_selected(matches!(self.current, Button::Yes));
        self.no_button
            .set_selected(matches!(self.current, Button::No));

        // Create areas for header, main, and footer
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        WindowUtils::render_header("Trust certificate", header_area, buffer);
        TrustTlsWindow::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl TrustTlsWindow {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ⇄ to switch, ↵ to confirm.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        let area = WindowUtils::render_background(area, buffer);

        // TODO: Maybe this can be done with one Paragraph instead of three?
        let [title_area, _, host_area, _, fingerprint_area, button_area] = Layout::vertical([
            Constraint::Length(1), // Title
            Constraint::Length(1),
            Constraint::Length(1), // Host
            Constraint::Length(1),
            Constraint::Fill(2), // Fingerprint
            Constraint::Length(3),
        ])
        .areas(area);

        Paragraph::new("A new certificate has been detected. This certificate is currently untrusted. Would you like to trust it?").blue().bold().centered().render(title_area, buffer);
        Paragraph::new(self.request.get_host().host.to_string())
            .cyan()
            .bold()
            .centered()
            .render(host_area, buffer);
        Paragraph::new(format!("{}", self.request.get_host()))
            .gray()
            .centered()
            .render(fingerprint_area, buffer);

        let [_, yes_area, _, no_area, _] = Layout::horizontal([
            Constraint::Fill(7),
            Constraint::Fill(6),
            Constraint::Fill(1),
            Constraint::Fill(6),
            Constraint::Fill(7),
        ])
        .areas(button_area);

        self.yes_button.render(yes_area, buffer);
        self.no_button.render(no_area, buffer);
    }
}
