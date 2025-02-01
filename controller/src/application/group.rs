use anyhow::Result;
use common::allocator::NumberAllocator;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    node::LifecycleStatus,
    server::{manager::ServerManager, Resources, Spec},
};

pub mod manager;

pub struct Group {
    /* Settings */
    name: String,
    status: LifecycleStatus,

    /* Where? */
    nodes: Vec<String>,
    constraints: StartConstraints,
    scaling: ScalingPolicy,

    /* How? */
    resources: Resources,
    spec: Spec,

    /* What do i need to know? */
    id_allocator: NumberAllocator<usize>,
    servers: Vec<AssociatedUnit>,
}

impl Group {
    pub fn tick(&mut self, servers: &ServerManager) -> Result<()> {
        if self.status == LifecycleStatus::Inactive {
            // Do not tick this group because it is inactive
            return Ok(());
        }

        let mut target_count = self.constraints.minimum;

        // Apply scling policy
        if self.scaling.enabled {
            self.servers.retain(|server| match server {
                AssociatedUnit::Active(server) => servers.get_server(server).map_or(false, |server| {
                    if server.get_connected_users() as f32 / self.spec.get_max_players() as f32 >= self.scaling.start_threshold {
                        target_count += 1;
                    }
                    true
                }),
                _ => true,
            });
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct StartConstraints {
    minimum: u32,
    maximum: u32,
    priority: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ScalingPolicy {
    enabled: bool,
    start_threshold: f32,
    stop_empty_units: bool,
}

pub enum AssociatedUnit {
    Queueing(Uuid),
    Active(Uuid),
}
