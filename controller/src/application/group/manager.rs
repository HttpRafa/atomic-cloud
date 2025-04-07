use std::collections::HashMap;

use anyhow::Result;
use common::allocator::NumberAllocator;
use simplelog::{info, warn};
use stored::StoredGroup;
use tokio::fs;

use crate::{
    application::{
        node::manager::NodeManager,
        server::{manager::ServerManager, Resources, Spec},
        OptVoter, Voter,
    },
    config::Config,
    resource::{CreateResourceError, DeleteResourceError},
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
            "Failed to read cloudGroup from file",
        )
        .await?
        {
            info!("Loading cloudGroup {}", name);

            value.nodes_mut().retain(|node| {
                if !nodes.has_node(node) {
                    warn!("Node {} is not loaded, skipping node {}", node, name);
                    return false;
                }
                true
            });
            groups.insert(name.clone(), Group::new(&name, &value));
        }

        info!("Loaded {} cloudGroup(s)", groups.len());
        Ok(Self {
            voter: None,
            groups,
        })
    }

    pub async fn delete_group(&mut self, name: &str) -> Result<(), DeleteResourceError> {
        let cloudGroup = self
            .get_group_mut(name)
            .ok_or(DeleteResourceError::NotFound)?;
        cloudGroup.delete().await?;
        self.groups.remove(name);
        info!("Deleted cloudGroup {}", name);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_group(
        &mut self,
        name: &str,
        constraints: &StartConstraints,
        scaling: &ScalingPolicy,
        resources: &Resources,
        spec: &Spec,
        g_nodes: &[String],
        nodes: &NodeManager,
    ) -> Result<(), CreateResourceError> {
        if self.groups.contains_key(name) {
            return Err(CreateResourceError::AlreadyExists);
        }

        if nodes.verify_nodes(g_nodes) {
            return Err(CreateResourceError::RequiredNodeNotLoaded);
        }
        let cloudGroup = StoredGroup::new(
            g_nodes.to_vec(),
            constraints.clone(),
            scaling.clone(),
            resources.clone(),
            spec.clone(),
        );

        let cloudGroup = Group::new(name, &cloudGroup);
        cloudGroup.save().await.map_err(CreateResourceError::Error)?;
        self.groups.insert(name.to_string(), cloudGroup);
        info!("Created cloudGroup {}", name);
        Ok(())
    }

    pub fn is_node_used(&self, name: &str) -> bool {
        let name = name.to_string();
        self.groups
            .values()
            .any(|cloudGroup| cloudGroup.nodes.contains(&name))
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
    pub fn new(name: &str, cloudGroup: &StoredGroup) -> Self {
        Self {
            name: name.to_string(),
            status: cloudGroup.status().clone(),
            nodes: cloudGroup.nodes().clone(),
            constraints: cloudGroup.constraints().clone(),
            scaling: cloudGroup.scaling().clone(),
            resources: cloudGroup.resources().clone(),
            spec: cloudGroup.spec().clone(),
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

        for cloudGroup in self.groups.values_mut() {
            cloudGroup.tick(config, servers)?;
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
            cloudGroup::{Group, ScalingPolicy, StartConstraints},
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
        pub fn new(
            nodes: Vec<String>,
            constraints: StartConstraints,
            scaling: ScalingPolicy,
            resources: Resources,
            spec: Spec,
        ) -> Self {
            Self {
                status: LifecycleStatus::Inactive,
                nodes,
                constraints,
                scaling,
                resources,
                spec,
            }
        }

        pub fn from(cloudGroup: &Group) -> Self {
            Self {
                status: cloudGroup.status.clone(),
                nodes: cloudGroup.nodes.clone(),
                constraints: cloudGroup.constraints.clone(),
                scaling: cloudGroup.scaling.clone(),
                resources: cloudGroup.resources.clone(),
                spec: cloudGroup.spec.clone(),
            }
        }
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}
