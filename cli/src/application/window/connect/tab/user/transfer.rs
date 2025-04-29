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
use tonic::async_trait;

use crate::application::{
    network::{
        connection::{task::EmptyTask, EstablishedConnection},
        proto::manage::{
            transfer::{target::Type, Target, TransferReq},
            user,
        },
    },
    util::status::{Status, StatusDisplay},
    window::{
        connect::tab::{
            global::delete::AUTO_CLOSE_AFTER,
            util::{fetch::FetchWindow, multi_select::MultiSelectWindow, select::SelectWindow},
        },
        StackBatcher, Window,
    },
    State,
};

pub struct TransferUserTab {
    /* Network */
    request: EmptyTask,

    /* Window */
    status: StatusDisplay,
}

impl TransferUserTab {
    /// Creates a new transfer user tab.
    /// This function will create a window stack to get the required information to transfer the users.
    pub fn new_stack(connection: Arc<EstablishedConnection>) -> FetchWindow<Vec<user::Item>> {
        FetchWindow::new(
            connection.get_users(),
            connection,
            move |users, connection: Arc<EstablishedConnection>, stack, _| {
                stack.push(MultiSelectWindow::new("Select the user/s you want to transfer", users, move |users, stack, _| {
                    stack.push(SelectWindow::new("Where would you like to transfer the users?",
                        vec![Type::Server, Type::Group, Type::Fallback],
                        move |typ, stack, _| {
                            match typ {
                                Type::Server => stack.push(FetchWindow::new(
                                    connection.get_servers(),
                                    connection,
                                    move |servers, connection, stack, _| {
                                        stack.push(SelectWindow::new("To which server do you want to transfer the users?",
                                            servers,
                                            move |server, stack, _| {
                                                stack.push(TransferUserTab::new(
                                                    &connection,
                                                    users,
                                                    Type::Server,
                                                    Some(server.id),
                                                ));
                                                Ok(())
                                            },
                                        ));
                                        Ok(())
                                    },
                                )),
                                Type::Group => stack.push(FetchWindow::new(
                                    connection.get_groups(),
                                    connection,
                                    move |groups, connection, stack, _| {
                                        stack.push(SelectWindow::new("To which group do you want to transfer the users?",
                                            groups,
                                            move |group, stack, _| {
                                                stack.push(TransferUserTab::new(
                                                    &connection,
                                                    users,
                                                    Type::Group,
                                                    Some(group.name),
                                                ));
                                                Ok(())
                                            },
                                        ));
                                        Ok(())
                                    },
                                )),
                                Type::Fallback => stack.push(TransferUserTab::new(
                                    &connection,
                                    users,
                                    Type::Fallback,
                                    None,
                                )),
                            }
                            Ok(())
                        },
                    ));
                    Ok(())
                }));
                Ok(())
            },
        )
    }

    pub fn new<T>(
        connection: &Arc<EstablishedConnection>,
        users: T,
        typ: Type,
        target: Option<String>,
    ) -> Self
    where
        T: IntoIterator<Item = user::Item>,
    {
        Self {
            request: connection.transfer_users(TransferReq {
                ids: users.into_iter().map(|user| user.id).collect(),
                target: Some(Target {
                    r#type: typ as i32,
                    target,
                }),
            }),
            status: StatusDisplay::new_with_startpoint(Status::Loading, "Submitting request..."),
        }
    }
}

#[async_trait]
impl Window for TransferUserTab {
    async fn init(&mut self, _stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        Ok(())
    }

    async fn tick(&mut self, stack: &mut StackBatcher, _state: &mut State) -> Result<()> {
        // Network connection
        match self.request.get_now().await {
            Ok(Some(Ok(()))) => {
                self.status.change_with_startpoint(
                    Status::Successful,
                    "Sucessfully submitted transfer requests",
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

impl Widget for &mut TransferUserTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        TransferUserTab::render_footer(footer_area, buffer);

        self.render_body(main_area, buffer);
    }
}

impl TransferUserTab {
    fn render_footer(area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ↵ to select, Esc to close tab.")
            .centered()
            .render(area, buffer);
    }

    fn render_body(&mut self, area: Rect, buffer: &mut Buffer) {
        self.status.render_in_center(area, buffer);
    }
}

impl Display for user::Item {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} ({})", self.name, self.id)
    }
}

impl Display for Type {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Group => write!(formatter, "Group"),
            Type::Server => write!(formatter, "Server"),
            Type::Fallback => write!(formatter, "Fallback"),
        }
    }
}
