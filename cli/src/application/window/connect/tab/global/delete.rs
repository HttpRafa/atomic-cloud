use std::{
    fmt::{Display, Formatter},
    sync::Arc,
    time::Duration,
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
    network::{
        connection::{task::EmptyTask, EstablishedConnection},
        proto::manage::resource::{Category, DelReq},
    },
    util::{
        status::{Status, StatusDisplay},
        TEXT_FG_COLOR,
    },
    window::{
        connect::tab::util::{fetch::FetchWindow, select::SelectWindow},
        StackBatcher, Window,
    },
    State,
};

pub const AUTO_CLOSE_AFTER: Duration = Duration::from_secs(15);

pub struct DeleteTab {
    /* Network */
    request: EmptyTask,

    /* Window */
    status: StatusDisplay,
}

impl DeleteTab {
    /// Creates a new get delete tab.
    /// This function will create a window stack to get the required information to display the server.
    /// What do we want to delete? -> Fetch options -> Select for each category -> Push delete tab
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> SelectWindow<'static, Category> {
        SelectWindow::new(
            vec![Category::Server, Category::Node, Category::Group],
            move |category, stack, _| {
                match category {
                    Category::Server => stack.push(FetchWindow::new(
                        connection.get_servers(),
                        connection.clone(),
                        move |servers, connection, stack, _| {
                            stack.push(SelectWindow::new_with_confirmation(
                                servers,
                                move |server, stack, _| {
                                    stack.push(DeleteTab::new(
                                        connection.clone(),
                                        Category::Server,
                                        server.id,
                                    ));
                                    Ok(())
                                },
                            ));
                            Ok(())
                        },
                    )),
                    Category::Node => stack.push(FetchWindow::new(
                        connection.get_nodes(),
                        connection.clone(),
                        move |nodes, connection, stack, _| {
                            stack.push(SelectWindow::new_with_confirmation(
                                nodes,
                                move |node, stack, _| {
                                    stack.push(DeleteTab::new(
                                        connection.clone(),
                                        Category::Node,
                                        node.name,
                                    ));
                                    Ok(())
                                },
                            ));
                            Ok(())
                        },
                    )),
                    Category::Group => stack.push(FetchWindow::new(
                        connection.get_groups(),
                        connection.clone(),
                        move |groups, connection, stack, _| {
                            stack.push(SelectWindow::new_with_confirmation(
                                groups,
                                move |group, stack, _| {
                                    stack.push(DeleteTab::new(
                                        connection.clone(),
                                        Category::Group,
                                        group.name,
                                    ));
                                    Ok(())
                                },
                            ));
                            Ok(())
                        },
                    )),
                }
                Ok(())
            },
        )
    }

    pub fn new(connection: Arc<EstablishedConnection>, category: Category, id: String) -> Self {
        Self {
            request: connection.delete_resource(DelReq {
                category: category as i32,
                id,
            }),
            status: StatusDisplay::new(Status::Loading, "Deleting resource from controller..."),
        }
    }
}

#[async_trait]
impl Window for DeleteTab {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Network connection
        match self.request.get_now().await {
            Ok(Some(Ok(()))) => {
                self.status.change_with_startpoint(
                    Status::Successful,
                    "Sucessfully deleted resource from controller",
                );
            }
            Err(error) | Ok(Some(Err(error))) => {
                self.status
                    .change(Status::Fatal, format!("{}", error.root_cause()));
            }
            _ => {}
        }

        // UI
        self.status.next();
        if self.status.is_successful() && self.status.elapsed() > AUTO_CLOSE_AFTER {
            stack.close_tab();
        }
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
                stack.close_tab();
            }
        }
        Ok(())
    }

    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        Widget::render(self, area, buffer);
    }
}

impl Widget for &mut DeleteTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        DeleteTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl DeleteTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        self.status.render_in_center(area, buffer);
    }
}

impl From<&Category> for ListItem<'_> {
    fn from(category: &Category) -> Self {
        ListItem::new(Line::styled(format!(" {category}"), TEXT_FG_COLOR))
    }
}

impl Display for Category {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Group => write!(formatter, "Group"),
            Category::Node => write!(formatter, "Node"),
            Category::Server => write!(formatter, "Server"),
        }
    }
}
