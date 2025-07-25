use std::{
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use anyhow::Result;
use common::network::HostAndPort;
use getset::Getters;
use simplelog::{info, warn};
use tokio::{task::JoinHandle, time::Instant};
use uuid::Uuid;

use crate::{
    application::{
        OptVoter, Shared, Voter, group::manager::GroupManager, node::manager::NodeManager,
        user::manager::UserManager,
    },
    config::Config,
};

use super::{
    NameAndUuid, Resources, Server, Specification, State, guard::WeakGuard, screen::BoxedScreen,
};

mod action;
mod restart;
mod start;
mod stop;

pub struct ServerManager {
    voter: OptVoter,

    /* Servers */
    servers: HashMap<Uuid, Server>,

    /* Requests */
    start_requests: BinaryHeap<StartRequest>,
    restart_requests: Vec<RestartRequest>,
    stop_requests: Vec<StopRequest>,
}

impl ServerManager {
    pub fn init() -> Self {
        Self {
            voter: None,
            servers: HashMap::new(),
            start_requests: BinaryHeap::new(),
            restart_requests: vec![],
            stop_requests: vec![],
        }
    }

    pub fn is_node_used(&self, name: &str) -> bool {
        self.servers.values().any(|server| server.node == name)
    }

    pub fn find_fallback_server(&self, ignore: &Uuid) -> Option<&Server> {
        self.servers
            .values()
            .filter(|server| {
                server.id.uuid() != ignore
                    && server.ready
                    && server.state == State::Running
                    && server.allocation.specification.fallback.enabled
            })
            .max_by_key(|server| server.allocation.specification.fallback.priority)
    }

    pub fn get_servers(&self) -> Vec<&Server> {
        self.servers.values().collect()
    }

    pub fn get_server_from_name(&self, name: &str) -> Option<&Server> {
        self.servers
            .values()
            .find(|server| server.id().name() == name)
    }

    pub fn get_server(&self, uuid: &Uuid) -> Option<&Server> {
        self.servers.get(uuid)
    }
    pub fn get_server_mut(&mut self, uuid: &Uuid) -> Option<&mut Server> {
        self.servers.get_mut(uuid)
    }

    pub fn resolve_server(&self, uuid: &Uuid) -> Option<NameAndUuid> {
        self.servers.get(uuid).map(|server| server.id.clone())
    }

    pub fn cancel_start(&mut self, uuid: &Uuid) {
        self.start_requests
            .retain(|request| request.id.uuid() != uuid);
    }

    pub fn schedule_start(&mut self, request: StartRequest) {
        if self.voter.is_some() {
            warn!(
                "Ignoring start request for server {} as the server manager is shutting down.",
                request.id
            );
            return;
        }
        self.start_requests.push(request);
    }
    pub fn _schedule_restart(&mut self, request: RestartRequest) {
        if self.restart_requests.contains(&request) {
            return;
        }
        self.restart_requests.push(request);
    }
    pub fn schedule_stop(&mut self, request: StopRequest) {
        if self.stop_requests.contains(&request) {
            return;
        }
        self.stop_requests.push(request);
    }
    pub fn schedule_stops(&mut self, requests: Vec<StopRequest>) {
        for request in requests {
            self.schedule_stop(request);
        }
    }
}

// Ticking
impl ServerManager {
    #[allow(clippy::too_many_lines)]
    pub async fn tick(
        &mut self,
        config: &Config,
        nodes: &NodeManager,
        groups: &mut GroupManager,
        users: &mut UserManager,
        shared: &Arc<Shared>,
    ) -> Result<()> {
        // Check health of servers
        for server in self.servers.values() {
            if server.heart.is_dead() {
                match server.state {
                    State::Starting | State::Restarting => {
                        warn!(
                            "Unit {} failed to establish online status within the expected startup time of {:.2?}.",
                            server.id,
                            config.restart_timeout()
                        );
                    }
                    State::Running => {
                        warn!(
                            "Server {} has not checked in for {:.2?}, indicating a potential error.",
                            server.id, server.heart.timeout
                        );
                    }
                    State::Stopping => continue, // We ignore that the server has not checked in because he is stopping
                }
                self.restart_requests
                    .push(RestartRequest::new(None, server.id().clone()));
            }
        }

        // Stop all servers that have been requested to stop
        {
            let mut requests = Vec::with_capacity(self.stop_requests.len());
            for mut request in self.stop_requests.drain(..) {
                if Self::handle_stop_request(
                    &mut request,
                    &mut self.servers,
                    nodes,
                    groups,
                    users,
                    shared,
                )
                .await?
                {
                    requests.push(request);
                }
            }
            self.stop_requests.extend(requests);
        }

        // Restart all servers that have been requested to restart
        {
            let mut requests = Vec::with_capacity(self.restart_requests.len());
            for mut request in self.restart_requests.drain(..) {
                if Self::handle_restart_request(&mut request, &mut self.servers, config, nodes)
                    .await?
                {
                    requests.push(request);
                }
            }
            self.restart_requests.extend(requests);
        }

        // Start all servers that have been requested to start
        {
            let mut requests = Vec::with_capacity(self.start_requests.len());
            for mut request in self.start_requests.drain_sorted() {
                if Self::handle_start_request(
                    &mut request,
                    &mut self.servers,
                    config,
                    nodes,
                    groups,
                    shared,
                )
                .await?
                {
                    requests.push(request);
                }
            }
            self.start_requests.extend(requests);
        }

        if let Some(voter) = &mut self.voter
            && self.servers.is_empty()
            && voter.vote()
        {
            info!("All servers have been stopped. Ready to stop...");
        }

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn shutdown(&mut self, voter: Voter) -> Result<()> {
        self.voter = Some(voter);

        info!("Canceling all start requests...");
        self.start_requests.clear();
        self.restart_requests.clear();
        info!("Shutting down all servers...");
        let mut requests = Vec::with_capacity(self.servers.len());
        for server in self.servers.values() {
            requests.push(StopRequest::new(None, server.id().clone()));
        }
        self.schedule_stops(requests);

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Getters)]
pub struct StartRequest {
    /* Request */
    when: Option<Instant>,

    /* Server */
    #[getset(get = "pub")]
    id: NameAndUuid,
    #[getset(get = "pub")]
    group: Option<String>,
    #[getset(get = "pub")]
    nodes: Vec<String>,
    #[getset(get = "pub")]
    resources: Resources,
    #[getset(get = "pub")]
    specification: Specification,
    #[getset(get = "pub")]
    priority: i32,

    /* Stage */
    #[getset(get = "pub")]
    stage: StartStage,
}

#[derive(Getters)]
pub struct RestartRequest {
    /* Request */
    when: Option<Instant>,
    server: NameAndUuid,

    /* Stage */
    #[getset(get = "pub")]
    stage: RestartStage,
}

#[derive(Getters)]
pub struct StopRequest {
    /* Request */
    when: Option<Instant>,
    server: NameAndUuid,

    /* Stage */
    #[getset(get = "pub")]
    stage: StopStage,
}

enum RestartStage {
    Queued,
    Running(JoinHandle<Result<()>>),
}

enum StopStage {
    Queued,
    Freeing(JoinHandle<Result<()>>),
    Running(JoinHandle<Result<()>>, WeakGuard),
    Stopping(WeakGuard),
}

enum StartStage {
    Queued,
    Allocating(usize, JoinHandle<Result<Vec<HostAndPort>>>),
    Creating(JoinHandle<Result<BoxedScreen>>),
}

impl StartRequest {
    pub fn new(
        when: Option<Instant>,
        priority: i32,
        name: String,
        group: Option<String>,
        nodes: &[String],
        resources: &Resources,
        specification: &Specification,
    ) -> Self {
        Self {
            id: NameAndUuid::generate(name),
            when,
            priority,
            group,
            nodes: nodes.to_vec(),
            resources: resources.clone(),
            specification: specification.clone(),
            stage: StartStage::Queued,
        }
    }
}

impl RestartRequest {
    pub fn new(when: Option<Instant>, server: NameAndUuid) -> Self {
        Self {
            when,
            server,
            stage: RestartStage::Queued,
        }
    }
}

impl StopRequest {
    pub fn new(when: Option<Instant>, server: NameAndUuid) -> Self {
        Self {
            when,
            server,
            stage: StopStage::Queued,
        }
    }
}

impl PartialEq for StopRequest {
    fn eq(&self, other: &Self) -> bool {
        self.server.uuid == other.server.uuid
    }
}

impl PartialEq for RestartRequest {
    fn eq(&self, other: &Self) -> bool {
        self.server.uuid == other.server.uuid
    }
}

impl Ord for StartRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}
impl PartialOrd for StartRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for StartRequest {}
impl PartialEq for StartRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
