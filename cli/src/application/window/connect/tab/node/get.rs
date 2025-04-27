use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

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
    network::{connection::EstablishedConnection, proto::manage::node},
    util::{fancy_toml::FancyToml, TEXT_FG_COLOR},
    window::{
        connect::tab::util::{fetch::FetchWindow, select::SelectWindow},
        StackBatcher, Window,
    },
    State,
};

pub struct GetNodeTab {
    /* Connection */
    connection: Arc<EstablishedConnection>,
    node: node::Detail,

    /* Lines */
    lines: Vec<Line<'static>>,
}

impl GetNodeTab {
    /// Creates a new get node tab.
    /// This function will create a window stack to get the required information to display the node.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> FetchWindow<Vec<node::Short>> {
        FetchWindow::new(
            connection.get_nodes(),
            connection,
            move |nodes, connection: Arc<EstablishedConnection>, stack, _| {
                stack.push(SelectWindow::new(nodes, move |node, stack, _| {
                    stack.push(FetchWindow::new(
                        connection.get_node(&node.name),
                        connection.clone(),
                        move |node, connection, stack, _| {
                            stack.push(GetNodeTab::new(connection.clone(), node));
                            Ok(())
                        },
                    ));
                    Ok(())
                }));
                Ok(())
            },
        )
    }

    pub fn new(connection: Arc<EstablishedConnection>, node: node::Detail) -> Self {
        Self {
            connection,
            node,
            lines: vec![],
        }
    }
}

#[async_trait]
impl Window for GetNodeTab {
    async fn init(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Compute the lines
        if let Ok(toml) = toml::to_string_pretty(&self.node) {
            self.lines.extend(FancyToml::to_lines(&toml));
        }

        // Change the title
        stack.rename_tab(&self.node.name);

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
        if let Event::Key(event) = event {
            if event.kind != KeyEventKind::Press {
                return Ok(());
            }
            if event.code == KeyCode::Esc {
                stack.close_tab();
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut GetNodeTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        GetNodeTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl GetNodeTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines.clone()).render(area, buffer);
    }
}

impl From<&node::Short> for ListItem<'_> {
    fn from(node: &node::Short) -> Self {
        ListItem::new(Line::styled(format!(" {node}"), TEXT_FG_COLOR))
    }
}

impl Display for node::Short {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
