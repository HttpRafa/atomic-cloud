use std::collections::{HashMap, VecDeque};

use anyhow::Result;
use getset::Getters;
use tokio::time::Instant;
use uuid::Uuid;

use super::{Resources, Server, Spec};

pub struct ServerManager {
    /* Servers */
    servers: HashMap<Uuid, Server>,

    /* Requests */
    start_requests: VecDeque<StartRequest>,
    stop_requests: VecDeque<StopRequest>,
}

impl ServerManager {
    pub async fn init() -> Result<Self> {
        Ok(Self {
            servers: HashMap::new(),
            start_requests: VecDeque::new(),
            stop_requests: VecDeque::new(),
        })
    }

    pub fn get_server(&self, uuid: &Uuid) -> Option<&Server> {
        self.servers.get(uuid)
    }
    pub fn get_server_mut(&mut self, uuid: &Uuid) -> Option<&mut Server> {
        self.servers.get_mut(uuid)
    }

    pub fn schedule_start(&mut self, request: StartRequest) {
        self.start_requests.push_back(request);
    }
    pub fn schedule_stop(&mut self, request: StopRequest) {
        self.stop_requests.push_back(request);
    }
    pub fn schedule_stops(&mut self, requests: Vec<StopRequest>) {
        for request in requests.into_iter() {
            self.stop_requests.push_back(request);
        }
    }
}

// Ticking
impl ServerManager {
    pub async fn tick(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Getters)]
pub struct StartRequest {
    /* Request */
    #[getset(get = "pub")]
    uuid: Uuid,
    when: Option<Instant>,

    /* Server */
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    group: Option<String>,
    #[getset(get = "pub")]
    nodes: Vec<String>,
    #[getset(get = "pub")]
    resources: Resources,
    #[getset(get = "pub")]
    spec: Spec,
    priority: i32,
}

pub struct StopRequest {
    when: Option<Instant>,
    server: Uuid,
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
            uuid: Uuid::new_v4(),
            when,
            priority,
            name,
            group,
            nodes: nodes.to_vec(),
            resources: resources.clone(),
            spec: spec.clone(),
        }
    }
}

impl StopRequest {
    pub fn new(when: Option<Instant>, server: &Uuid) -> Self {
        Self {
            when,
            server: *server,
        }
    }
}
