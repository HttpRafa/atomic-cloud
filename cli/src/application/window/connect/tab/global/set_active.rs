use std::{
    fmt::{Display, Formatter},
    sync::Arc,
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
    network::{
        connection::{task::EmptyTask, EstablishedConnection},
        proto::manage::resource::{Category, SetReq},
    },
    util::status::{Status, StatusDisplay},
    window::{
        connect::tab::util::{
            fetch::FetchWindow, multi_select::MultiSelectWindow, select::SelectWindow,
        },
        StackBatcher, Window,
    },
    State,
};

use super::delete::AUTO_CLOSE_AFTER;

// This needs to pub because of the function `new_stack` in the `SetActiveTab` struct
pub enum ActiveState {
    Active,
    Inactive,
}

pub struct SetActiveTab {
    /* Network */
    requests: Vec<(StatusDisplay, EmptyTask)>,

    /* Window */
    finished: Option<Instant>,
}

impl SetActiveTab {
    /// Creates a new set active tab.
    /// This function will create a window stack to get the required information to set the active status of a node/group.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> SelectWindow<'static, ActiveState> {
        SelectWindow::new(
            "Do you want to active or deactive the resources?",
            vec![ActiveState::Active, ActiveState::Inactive],
            move |state, stack, _| {
                stack.push(SelectWindow::new(
                    "What type of resource do you want to change?",
                    vec![Category::Node, Category::Group],
                    move |category, stack, _| {
                        match category {
                            Category::Node => stack.push(FetchWindow::new(
                                connection.get_nodes(),
                                connection,
                                move |nodes, connection, stack, _| {
                                    stack.push(MultiSelectWindow::new(
                                        "Select the node/s you want to change",
                                        nodes,
                                        move |nodes, stack, _| {
                                            stack.push(SetActiveTab::new(
                                                &connection,
                                                matches!(state, ActiveState::Active),
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
                                        "Select the group/s you want to change",
                                        groups,
                                        move |groups, stack, _| {
                                            stack.push(SetActiveTab::new(
                                                &connection,
                                                matches!(state, ActiveState::Active),
                                                Category::Group,
                                                groups.into_iter().map(|group| group.name),
                                            ));
                                            Ok(())
                                        },
                                    ));
                                    Ok(())
                                },
                            )),
                            Category::Server => unimplemented!(), // Not implemented
                        }
                        Ok(())
                    },
                ));
                Ok(())
            },
        )
    }

    pub fn new<T>(
        connection: &Arc<EstablishedConnection>,
        active: bool,
        category: Category,
        ids: T,
    ) -> Self
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
                            format!("Setting status of resource({id})..."),
                        ),
                        connection.set_resource(SetReq {
                            active,
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
impl Window for SetActiveTab {
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
                        "Sucessfully set status of resource",
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

impl Widget for &mut SetActiveTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        SetActiveTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl SetActiveTab {
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

impl Display for ActiveState {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ActiveState::Active => write!(formatter, "Active"),
            ActiveState::Inactive => write!(formatter, "Inactive"),
        }
    }
}
