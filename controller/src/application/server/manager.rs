use std::{
    collections::{BinaryHeap, HashMap},
    mem,
};

use anyhow::Result;
use common::network::HostAndPort;
use getset::Getters;
use simplelog::{debug, warn};
use tokio::{task::JoinHandle, time::Instant};
use uuid::Uuid;

use crate::{
    application::{
        auth::validator::WrappedAuthValidator, group::manager::GroupManager,
        node::manager::NodeManager, user::manager::UserManager,
    },
    config::Config,
};

use super::{NameAndUuid, Resources, Server, Spec, State};

mod action;

pub struct ServerManager {
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
            servers: HashMap::new(),
            start_requests: BinaryHeap::new(),
            restart_requests: vec![],
            stop_requests: vec![],
        }
    }

    pub fn get_server(&self, uuid: &Uuid) -> Option<&Server> {
        self.servers.get(uuid)
    }
    pub fn get_server_mut(&mut self, uuid: &Uuid) -> Option<&mut Server> {
        self.servers.get_mut(uuid)
    }

    pub fn schedule_start(&mut self, request: StartRequest) {
        self.start_requests.push(request);
    }
    pub fn schedule_restart(&mut self, request: RestartRequest) {
        self.restart_requests.push(request);
    }
    pub fn schedule_stop(&mut self, request: StopRequest) {
        self.stop_requests.push(request);
    }
    pub fn schedule_stops(&mut self, requests: Vec<StopRequest>) {
        self.stop_requests.extend(requests);
    }
}

// Ticking
impl ServerManager {
    pub async fn tick(
        &mut self,
        config: &Config,
        nodes: &NodeManager,
        groups: &mut GroupManager,
        users: &mut UserManager,
        validator: &WrappedAuthValidator,
    ) -> Result<()> {
        // Check health of servers
        for server in self.servers.values() {
            if server.health.is_dead() {
                match server.state {
                    State::Starting | State::Running => {
                        warn!("Unit {} failed to establish online status within the expected startup time of {:.2?}.", server.id, config.restart_timeout());
                    }
                    _ => {
                        warn!("Server {} has not checked in for {:.2?}, indicating a potential error.", server.id, server.health.timeout);
                    }
                }
                self.restart_requests
                    .push(RestartRequest::new(None, server.id()));
            }
        }

        // Stop all servers that have been requested to stop
        let mut requests = vec![];
        while let Some(mut request) = self.stop_requests.pop() {
            if let Some(when) = request.when {
                if when > Instant::now() {
                    requests.push(request);
                    continue;
                }
            }

            let mut reinsert = false;
            request.stage = match mem::replace(&mut request.stage, ActionStage::Queued) {
                ActionStage::Queued => {
                    debug!("Freeing resources for server {}", request.server);
                    match Self::free(&request, &mut self.servers, nodes) {
                        Ok(handle) => {
                            reinsert = true;
                            ActionStage::Freeing(handle)
                        }
                        Err(error) => {
                            warn!("Failed to free server {}: {}", request.server, error);
                            ActionStage::Failed
                        }
                    }
                }
                ActionStage::Freeing(handle) => {
                    if handle.is_finished() {
                        handle.await??;
                        debug!("Stopping server {}", request.server);
                        match Self::stop(
                            &request,
                            &mut self.servers,
                            nodes,
                            groups,
                            users,
                            validator,
                        )
                        .await
                        {
                            Ok(handle) => {
                                reinsert = true;
                                ActionStage::Running(handle)
                            }
                            Err(error) => {
                                warn!("Failed to stop server {}: {}", request.server, error);
                                ActionStage::Failed
                            }
                        }
                    } else {
                        reinsert = true;
                        ActionStage::Freeing(handle)
                    }
                }
                ActionStage::Running(handle) => {
                    if handle.is_finished() {
                        handle.await??;
                        debug!("Server {} has been stopped", request.server);
                        ActionStage::Finished
                    } else {
                        reinsert = true;
                        ActionStage::Running(handle)
                    }
                }
                _ => ActionStage::Finished,
            };
            if reinsert {
                requests.push(request);
            }
        }
        self.stop_requests.extend(requests);

        // Restart all servers that have been requested to restart
        let mut requests = vec![];
        while let Some(mut request) = self.restart_requests.pop() {
            if let Some(when) = request.when {
                if when > Instant::now() {
                    requests.push(request);
                    continue;
                }
            }

            let mut reinsert = false;
            request.stage = match mem::replace(&mut request.stage, ActionStage::Queued) {
                ActionStage::Queued => {
                    debug!("Restarting server {}", request.server);
                    match Self::restart(&request, &mut self.servers, config, nodes) {
                        Ok(handle) => {
                            reinsert = true;
                            ActionStage::Running(handle)
                        }
                        Err(error) => {
                            warn!("Failed to restart server {}: {}", request.server, error);
                            ActionStage::Failed
                        }
                    }
                }
                ActionStage::Running(handle) => {
                    if handle.is_finished() {
                        handle.await??;
                        debug!("Server {} has been restarted", request.server);
                        ActionStage::Finished
                    } else {
                        reinsert = true;
                        ActionStage::Running(handle)
                    }
                }
                _ => ActionStage::Finished,
            };
            if reinsert {
                requests.push(request);
            }
        }
        self.restart_requests.extend(requests);

        // Start all servers that have been requested to start
        let mut requests = vec![];
        while let Some(mut request) = self.start_requests.pop() {
            if request.nodes.is_empty() {
                warn!("Server {} has no nodes available to start on.", request.id);
                continue;
            }

            if let Some(when) = request.when {
                if when > Instant::now() {
                    requests.push(request);
                    continue;
                }
            }

            let mut reinsert = false;
            request.stage = match mem::replace(&mut request.stage, StartStage::Queued) {
                StartStage::Queued => {
                    debug!("Allocating resources for server {}", request.id);
                    match Self::allocate(0, &request, nodes) {
                        Ok(handle) => {
                            reinsert = true;
                            StartStage::Allocating((0, handle))
                        }
                        Err(error) => {
                            warn!(
                                "Failed to allocate resources for server {}: {}",
                                request.id, error
                            );
                            reinsert = false;
                            StartStage::Failed
                        }
                    }
                }
                StartStage::Allocating((index, handle)) => {
                    reinsert = true;
                    if handle.is_finished() {
                        let ports = handle.await?;
                        if let Ok(ports) = ports {
                            debug!("Creating server {}", request.id);
                            match Self::start(
                                index,
                                &request,
                                ports,
                                &mut self.servers,
                                config,
                                nodes,
                                groups,
                                validator,
                            )
                            .await
                            {
                                Ok(handle) => StartStage::Creating(handle),
                                Err(error) => {
                                    warn!("Failed to create server {}: {}", request.id, error);
                                    reinsert = false;
                                    StartStage::Failed
                                }
                            }
                        } else {
                            match Self::allocate(index + 1, &request, nodes) {
                                Ok(handle) => StartStage::Allocating((index + 1, handle)),
                                Err(error) => {
                                    warn!(
                                        "Failed to allocate resources for server {}: {}",
                                        request.id, error
                                    );
                                    reinsert = false;
                                    StartStage::Failed
                                }
                            }
                        }
                    } else {
                        StartStage::Allocating((index, handle))
                    }
                }
                StartStage::Creating(handle) => {
                    if handle.is_finished() {
                        handle.await??;
                        debug!("Server {} has been created", request.id);
                        StartStage::Started
                    } else {
                        reinsert = true;
                        StartStage::Creating(handle)
                    }
                }
                _ => StartStage::Started,
            };
            if reinsert {
                requests.push(request);
            }
        }
        self.start_requests.extend(requests);

        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
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
    spec: Spec,
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
    stage: ActionStage,
}

#[derive(Getters)]
pub struct StopRequest {
    /* Request */
    when: Option<Instant>,
    server: NameAndUuid,

    /* Stage */
    #[getset(get = "pub")]
    stage: ActionStage,
}

enum ActionStage {
    Queued,
    Freeing(JoinHandle<Result<()>>),
    Running(JoinHandle<Result<()>>),
    Finished,
    Failed,
}

enum StartStage {
    Queued,
    Allocating((usize, JoinHandle<Result<Vec<HostAndPort>>>)),
    Creating(JoinHandle<Result<()>>),
    Started,
    Failed,
}

impl StartRequest {
    pub fn new(
        when: Option<Instant>,
        priority: i32,
        name: String,
        group: Option<String>,
        nodes: &[String],
        resources: &Resources,
        spec: &Spec,
    ) -> Self {
        Self {
            id: NameAndUuid::generate(name),
            when,
            priority,
            group,
            nodes: nodes.to_vec(),
            resources: resources.clone(),
            spec: spec.clone(),
            stage: StartStage::Queued,
        }
    }
}

impl RestartRequest {
    pub fn new(when: Option<Instant>, server: &NameAndUuid) -> Self {
        Self {
            when,
            server: server.clone(),
            stage: ActionStage::Queued,
        }
    }
}

impl StopRequest {
    pub fn new(when: Option<Instant>, server: &NameAndUuid) -> Self {
        Self {
            when,
            server: server.clone(),
            stage: ActionStage::Queued,
        }
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
