use color_eyre::eyre::Result;
use ratatui::{DefaultTerminal, Frame};

pub struct Cli {
    running: bool,
}

impl Cli {
    pub fn new() -> Self {
        Self { running: true }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
        }
        Ok(())
    }

    fn render(&mut self, _frame: &mut Frame) {}
}
