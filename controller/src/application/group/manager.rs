use std::{collections::HashMap, vec};

use anyhow::Result;
use common::allocator::NumberAllocator;
use simplelog::{info, warn};
use stored::StoredGroup;
use tokio::fs;

use crate::{
    application::{
        node::manager::{DeleteResourceError, NodeManager},
        server::manager::ServerManager,
    },
    config::Config,
    storage::Storage,
};

use super::Group;

pub struct GroupManager {
    groups: HashMap<String, Group>,
}

impl GroupManager {
    pub async fn init(nodes: &NodeManager) -> Result<Self> {
        info!("Loading groups...");
        let mut groups = HashMap::new();

        let directory = Storage::groups_directory();
        if !directory.exists() {
            fs::create_dir_all(&directory).await?;
        }

        for (_, _, name, mut value) in Storage::for_each_content_toml::<StoredGroup>(
            &directory,
            "Failed to read group from file",
        )
        .await?
        {
            info!("Loading group {}", name);

            value.nodes_mut().retain(|node| {
                if !nodes.has_node(node) {
                    warn!("Node {} is not loaded, skipping node {}", node, name);
                    return false;
                }
                true
            });

            info!("Loaded group {}", name);
            groups.insert(name.clone(), Group::new(&name, value));
        }

        info!("Loaded {} group(s)", groups.len());
        Ok(Self { groups })
    }

    pub async fn delete_group(&mut self, name: &str) -> Result<(), DeleteResourceError> {
        let group = self
            .get_group_mut(name)
            .ok_or(DeleteResourceError::NotFound)?;
        group.delete().await?;
        self.groups.remove(name);
        info!("Deleted group {}", name);
        Ok(())
    }

    pub fn is_node_used(&self, name: &str) -> bool {
        let name = name.to_string();
        self.groups
            .values()
            .any(|group| group.nodes.contains(&name))
    }

    pub fn get_groups(&self) -> Vec<&Group> {
        self.groups.values().collect()
    }

    pub fn get_group(&self, name: &str) -> Option<&Group> {
        self.groups.get(name)
    }
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut Group> {
        self.groups.get_mut(name)
    }
}

impl Group {
    pub fn new(name: &str, group: StoredGroup) -> Self {
        Self {
            name: name.to_string(),
            status: group.status().clone(),
            nodes: group.nodes().clone(),
            constraints: group.constraints().clone(),
            scaling: group.scaling().clone(),
            resources: group.resources().clone(),
            spec: group.spec().clone(),
            id_allocator: NumberAllocator::new(1..usize::MAX),
            servers: vec![],
        }
    }
}

// Ticking
impl GroupManager {
    pub async fn tick(&mut self, config: &Config, servers: &mut ServerManager) -> Result<()> {
        for group in self.groups.values_mut() {
            group.tick(config, servers)?;
        }
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

pub(super) mod stored {
    use getset::{Getters, MutGetters};
    use serde::{Deserialize, Serialize};

    use crate::{
        application::{
            group::{Group, ScalingPolicy, StartConstraints},
            node::LifecycleStatus,
            server::{Resources, Spec},
        },
        storage::{LoadFromTomlFile, SaveToTomlFile},
    };

    #[derive(Serialize, Deserialize, Getters, MutGetters)]
    pub struct StoredGroup {
        /* Settings */
        #[getset(get = "pub", get_mut = "pub")]
        status: LifecycleStatus,

        /* Where? */
        #[getset(get = "pub", get_mut = "pub")]
        nodes: Vec<String>,
        #[getset(get = "pub", get_mut = "pub")]
        constraints: StartConstraints,
        #[getset(get = "pub", get_mut = "pub")]
        scaling: ScalingPolicy,

        /* How? */
        #[getset(get = "pub", get_mut = "pub")]
        resources: Resources,
        #[getset(get = "pub", get_mut = "pub")]
        spec: Spec,
    }

    impl StoredGroup {
        pub fn from(group: &Group) -> Self {
            Self {
                status: group.status.clone(),
                nodes: group.nodes.clone(),
                constraints: group.constraints.clone(),
                scaling: group.scaling.clone(),
                resources: group.resources.clone(),
                spec: group.spec.clone(),
            }
        }
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}
