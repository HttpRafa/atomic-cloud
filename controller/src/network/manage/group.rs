use anyhow::Result;
use tonic::{async_trait, Status};

use crate::{
    application::{
        group::{Group, ScalingPolicy, StartConstraints},
        server::{FallbackPolicy, Resources, Spec},
        Controller,
    },
    network::proto::{
        common::KeyValue,
        manage::{
            group::{Constraints, Item, List, Scaling},
            server::{self, Fallback},
        },
    },
    task::{BoxedAny, GenericTask, Task},
};

pub struct CreateGroupTask(
    pub String,
    pub StartConstraints,
    pub ScalingPolicy,
    pub Resources,
    pub Spec,
    pub Vec<String>,
);
pub struct GetGroupTask(pub String);
pub struct GetGroupsTask();

#[async_trait]
impl GenericTask for CreateGroupTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        if let Err(error) = controller
            .groups
            .create_group(
                &self.0,
                &self.1,
                &self.2,
                &self.3,
                &self.4,
                &self.5,
                &controller.nodes,
            )
            .await
        {
            return Task::new_err(error.into());
        }
        Task::new_empty()
    }
}

#[async_trait]
impl GenericTask for GetGroupTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let Some(group) = controller.groups.get_group(&self.0) else {
            return Task::new_err(Status::not_found("Group not found"));
        };

        Task::new_ok(Item::from(group))
    }
}

#[async_trait]
impl GenericTask for GetGroupsTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        Task::new_ok(List {
            groups: controller
                .groups
                .get_groups()
                .iter()
                .map(|group| group.name().clone())
                .collect(),
        })
    }
}

impl From<&Group> for Item {
    fn from(value: &Group) -> Self {
        Self {
            name: value.name().clone(),
            nodes: value.nodes().clone(),
            scaling: value.scaling().to_grpc(),
            constraints: Some(value.constraints().into()),
            resources: Some(value.resources().into()),
            spec: Some(value.spec().into()),
        }
    }
}

impl From<&StartConstraints> for Constraints {
    fn from(value: &StartConstraints) -> Self {
        Self {
            min: *value.minimum(),
            max: *value.maximum(),
            prio: *value.priority(),
        }
    }
}

impl ScalingPolicy {
    pub fn to_grpc(&self) -> Option<Scaling> {
        if *self.enabled() {
            Some(Scaling {
                start_threshold: *self.start_threshold(),
                stop_empty: *self.stop_empty_servers(),
            })
        } else {
            None
        }
    }
}

impl From<&Resources> for server::Resources {
    fn from(value: &Resources) -> Self {
        Self {
            memory: *value.memory(),
            swap: *value.swap(),
            cpu: *value.cpu(),
            io: *value.io(),
            disk: *value.disk(),
            ports: *value.ports(),
        }
    }
}

impl From<&Spec> for server::Spec {
    fn from(value: &Spec) -> Self {
        Self {
            img: value.image().clone(),
            max_players: *value.max_players(),
            settings: value
                .settings()
                .iter()
                .map(|(key, value)| KeyValue {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
            env: value
                .environment()
                .iter()
                .map(|(key, value)| KeyValue {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
            retention: Some(value.disk_retention().clone() as i32),
            fallback: value.fallback().to_grpc(),
        }
    }
}

impl FallbackPolicy {
    pub fn to_grpc(&self) -> Option<Fallback> {
        if *self.enabled() {
            Some(Fallback {
                prio: *self.priority(),
            })
        } else {
            None
        }
    }
}
