use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::Stylize, widgets::{Block, Borders, Paragraph, Widget}, Frame};
use tonic::async_trait;

use crate::application::{network::known_host::manager::TrustRequest, util::{button::SimpleButton, ERROR_COLOR, ERROR_SELECTED_COLOR, HEADER_STYLE, NORMAL_ROW_BG, OK_COLOR, OK_SELECTED_COLOR}, State};

use super::{create::CreateWindow, StackBatcher, Window, WindowUtils};

pub struct TrustTlsWindow {
    request: TrustRequest,

    yes_button: SimpleButton<'static>,
    no_button: SimpleButton<'static>,
}

impl TrustTlsWindow {
    pub fn new(request: TrustRequest) -> Self {
        Self { request, yes_button: 
            SimpleButton::new("Yes", "Trust this certificate", (OK_SELECTED_COLOR, OK_COLOR)), no_button: SimpleButton::new("No", "Do not trust this certificate", (ERROR_SELECTED_COLOR, ERROR_COLOR)) }
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
        _stack: &mut StackBatcher,
        _state: &mut State,
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
    fn render(self, area: Rect, buffer: &mut Buffer) {
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
        let block = Block::new()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);
        block.render(area, buffer);

        let [_, title_area, fingerprint_area, button_area, _] = Layout::vertical([
            Constraint::Length(2), // Empty space
            Constraint::Length(1), // Title
            Constraint::Fill(2), // Fingerprint
            Constraint::Length(3),
            Constraint::Fill(1), // Empty space
        ])
        .areas(area);

        let [_, yes_area, _, no_area, _] = Layout::horizontal([
            Constraint::Fill(7),
            Constraint::Fill(6),
            Constraint::Fill(1),
            Constraint::Fill(6),
            Constraint::Fill(7),
        ]).areas(button_area);
        
        self.yes_button.render(yes_area, buffer);
        self.no_button.render(no_area, buffer);
    }
}
