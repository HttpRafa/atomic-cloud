use crate::{
    application::server::{Server, State},
    network::proto::common::notify::{power_event, PowerEvent, ReadyEvent},
};

impl From<&Server> for PowerEvent {
    fn from(server: &Server) -> Self {
        Self {
            state: match server.state() {
                State::Starting => power_event::State::Start as i32,
                _ => power_event::State::Stop as i32,
            },
            name: server.id().name().clone(),
            node: server.node().clone(),
        }
    }
}

impl From<&Server> for ReadyEvent {
    fn from(server: &Server) -> Self {
        Self {
            ready: *server.ready(),
            name: server.id().name().clone(),
        }
    }
}
