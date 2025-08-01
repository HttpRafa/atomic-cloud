use std::collections::HashMap;

use anyhow::Result;
use common::allocator::NumberAllocator;
use simplelog::{debug, info, warn};
use stored::StoredGroup;
use tokio::fs;

use crate::{
    application::{
        OptVoter, Voter,
        node::manager::NodeManager,
        server::{Resources, Specification, manager::ServerManager},
    },
    config::Config,
    resource::{CreateResourceError, DeleteResourceError, UpdateResourceError},
    storage::Storage,
};

use super::{Group, ScalingPolicy, StartConstraints};

pub struct GroupManager {
    voter: OptVoter,

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
            groups.insert(name.clone(), Group::new(&name, &value));
        }

        info!("Loaded {} group(s)", groups.len());
        Ok(Self {
            voter: None,
            groups,
        })
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

    #[allow(clippy::too_many_arguments)]
    pub async fn create_group(
        &mut self,
        name: &str,
        constraints: &StartConstraints,
        scaling: &ScalingPolicy,
        resources: &Resources,
        specification: &Specification,
        g_nodes: &[String],
        nodes: &NodeManager,
    ) -> Result<(), CreateResourceError> {
        if self.groups.contains_key(name) {
            return Err(CreateResourceError::AlreadyExists);
        }

        if nodes.verify_nodes(g_nodes) {
            return Err(CreateResourceError::RequiredNodeNotLoaded);
        }
        let group = StoredGroup::new(
            g_nodes.to_vec(),
            constraints.clone(),
            scaling.clone(),
            resources.clone(),
            specification.clone(),
        );

        let group = Group::new(name, &group);
        group.save().await.map_err(CreateResourceError::Error)?;
        self.groups.insert(name.to_string(), group);
        info!("Created group {}", name);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_group(
        &mut self,
        name: &str,
        constraints: Option<&StartConstraints>,
        scaling: Option<&ScalingPolicy>,
        resources: Option<&Resources>,
        specification: Option<&Specification>,
        g_nodes: Option<&[String]>,
        nodes: &NodeManager,
    ) -> Result<&Group, UpdateResourceError> {
        let Some(group) = self.get_group_mut(name) else {
            return Err(UpdateResourceError::NotFound);
        };

        if let Some(g_nodes) = g_nodes {
            if nodes.verify_nodes(g_nodes) {
                return Err(UpdateResourceError::RequiredNodeNotLoaded);
            }
            group.set_nodes(g_nodes.to_vec());
        }
        if let Some(constraints) = constraints {
            group.set_constraints(constraints.clone());
        }
        if let Some(scaling) = scaling {
            group.set_scaling(scaling.clone());
        }
        if let Some(resources) = resources {
            group.set_resources(resources.clone());
        }
        if let Some(specification) = specification {
            group.set_specification(specification.clone());
        }
        group.save().await.map_err(UpdateResourceError::Error)?;
        debug!("Updated group {}", name);

        Ok(group)
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
    pub fn new(name: &str, group: &StoredGroup) -> Self {
        Self {
            name: name.to_string(),
            status: group.status().clone(),
            nodes: group.nodes().clone(),
            constraints: group.constraints().clone(),
            scaling: group.scaling().clone(),
            resources: group.resources().clone(),
            specification: group.specification().clone(),
            id_allocator: NumberAllocator::new(1..usize::MAX),
            servers: HashMap::new(),
        }
    }
}

// Ticking
impl GroupManager {
    pub fn tick(&mut self, config: &Config, servers: &mut ServerManager) -> Result<()> {
        if self.voter.is_some() {
            // Do not tick if we are shutting down
            return Ok(());
        }

        for group in self.groups.values_mut() {
            group.tick(config, servers)?;
        }
        Ok(())
    }

    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    pub fn shutdown(&mut self, mut voter: Voter) -> Result<()> {
        voter.vote();
        self.voter = Some(voter);
        Ok(())
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub fn cleanup(&mut self) -> Result<()> {
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
            server::{Resources, Specification},
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
        specification: Specification,
    }

    impl StoredGroup {
        pub fn new(
            nodes: Vec<String>,
            constraints: StartConstraints,
            scaling: ScalingPolicy,
            resources: Resources,
            specification: Specification,
        ) -> Self {
            Self {
                status: LifecycleStatus::Inactive,
                nodes,
                constraints,
                scaling,
                resources,
                specification,
            }
        }

        pub fn from(group: &Group) -> Self {
            Self {
                status: group.status.clone(),
                nodes: group.nodes.clone(),
                constraints: group.constraints.clone(),
                scaling: group.scaling.clone(),
                resources: group.resources.clone(),
                specification: group.specification.clone(),
            }
        }
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}
