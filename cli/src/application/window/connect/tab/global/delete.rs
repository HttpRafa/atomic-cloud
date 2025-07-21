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
    widgets::{Paragraph, Widget},
};
use tokio::time::Instant;
use tonic::async_trait;

use crate::application::{
    State,
    network::{
        connection::{EstablishedConnection, task::EmptyTask},
        proto::manage::resource::{Category, DelReq},
    },
    util::status::{Status, StatusDisplay},
    window::{
        StackBatcher, Window,
        connect::tab::util::{
            fetch::FetchWindow, multi_select::MultiSelectWindow, select::SelectWindow,
        },
    },
};

pub const AUTO_CLOSE_AFTER: Duration = Duration::from_secs(5);

pub struct DeleteTab {
    /* Network */
    requests: Vec<(StatusDisplay, EmptyTask)>,

    /* Window */
    finished: Option<Instant>,
}

impl DeleteTab {
    /// Creates a new get delete tab.
    /// This function will create a window stack to get the required information to display the server.
    /// What do we want to delete? -> Fetch options -> Select for each category -> Push delete tab
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> SelectWindow<'static, Category> {
        SelectWindow::new(
            "What type of resource do you want to delete?",
            vec![Category::Server, Category::Node, Category::Group],
            move |category, stack, _| {
                match category {
                    Category::Server => stack.push(FetchWindow::new(
                        connection.get_servers(),
                        connection,
                        move |servers, connection, stack, _| {
                            stack.push(MultiSelectWindow::new(
                                "Select the server/s you want to delete",
                                servers,
                                move |servers, stack, _| {
                                    stack.push(DeleteTab::new(
                                        &connection,
                                        Category::Server,
                                        servers.into_iter().map(|server| server.id),
                                    ));
                                    Ok(())
                                },
                            ));
                            Ok(())
                        },
                    )),
                    Category::Node => stack.push(FetchWindow::new(
                        connection.get_nodes(),
                        connection,
                        move |nodes, connection, stack, _| {
                            stack.push(MultiSelectWindow::new(
                                "Select the node/s you want to delete",
                                nodes,
                                move |nodes, stack, _| {
                                    stack.push(DeleteTab::new(
                                        &connection,
                                        Category::Node,
                                        nodes.into_iter().map(|node| node.name),
                                    ));
                                    Ok(())
                                },
                            ));
                            Ok(())
                        },
                    )),
                    Category::Group => stack.push(FetchWindow::new(
                        connection.get_groups(),
                        connection,
                        move |groups, connection, stack, _| {
                            stack.push(MultiSelectWindow::new(
                                "Select the group/s you want to delete",
                                groups,
                                move |groups, stack, _| {
                                    stack.push(DeleteTab::new(
                                        &connection,
                                        Category::Group,
                                        groups.into_iter().map(|group| group.name),
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

    pub fn new<T>(connection: &Arc<EstablishedConnection>, category: Category, ids: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        Self {
            requests: ids
                .into_iter()
                .map(|id| {
                    (
                        StatusDisplay::new_with_startpoint(
                            Status::Loading,
                            format!("Deleting resource({id}) from controller..."),
                        ),
                        connection.delete_resource(DelReq {
                            category: category as i32,
                            id,
                        }),
                    )
                })
                .collect(),
            finished: None,
        }
    }
}

#[async_trait]
impl Window for DeleteTab {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        for (status, request) in &mut self.requests {
            // Network connection
            match request.get_now().await {
                Ok(Some(Ok(()))) => {
                    status.change_with_startpoint(
                        Status::Successful,
                        "Sucessfully deleted resource from controller",
                    );
                }
                Err(error) | Ok(Some(Err(error))) => {
                    status.change(Status::Fatal, format!("{}", error.root_cause()));
                }
                _ => {}
            }

            // UI
            status.next();
        }

        if self
            .requests
            .iter()
            .all(|(status, _)| status.is_successful())
        {
            if let Some(instant) = &self.finished {
                if instant.elapsed() >= AUTO_CLOSE_AFTER {
                    stack.close_tab();
                }
            } else {
                self.finished = Some(Instant::now());
            }
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
                for (_, request) in &mut self.requests {
                    request.abort();
                }
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
        // Create a layout with the same number of areas as the requests + 2 for the top and bottom
        let mut layout = Vec::with_capacity(self.requests.len() + 2);
        layout.push(Constraint::Fill(1));
        layout.extend(std::iter::repeat_n(
            Constraint::Length(1),
            self.requests.len(),
        ));
        layout.push(Constraint::Fill(1));

        let areas = Layout::vertical(layout).split(area);
        self.requests
            .iter()
            .enumerate()
            .for_each(|(i, (status, _))| {
                status.render_in_center(areas[i + 1], buffer);
            });
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
