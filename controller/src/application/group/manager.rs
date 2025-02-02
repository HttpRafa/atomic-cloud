use std::{collections::HashMap, fs, vec};

use anyhow::Result;
use common::{allocator::NumberAllocator, file::for_each_content_toml};
use simplelog::{info, warn};
use stored::StoredGroup;

use crate::{
    application::{node::manager::NodeManager, server::manager::ServerManager},
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
            fs::create_dir_all(&directory)?;
        }

        for (_, _, name, mut value) in
            for_each_content_toml::<StoredGroup>(&directory, "Failed to read group from file")?
        {
            info!("Loading group {}", name);

            value.get_nodes_mut().retain(|node| {
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
}

impl Group {
    pub fn new(name: &str, group: StoredGroup) -> Self {
        Self {
            name: name.to_string(),
            status: group.get_status().clone(),
            nodes: group.get_nodes().clone(),
            constraints: group.get_constraints().clone(),
            scaling: group.get_scaling().clone(),
            resources: group.get_resources().clone(),
            spec: group.get_spec().clone(),
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

mod stored {
    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    use crate::application::{
        group::{ScalingPolicy, StartConstraints},
        node::LifecycleStatus,
        server::{Resources, Spec},
    };

    #[derive(Serialize, Deserialize)]
    pub struct StoredGroup {
        /* Settings */
        status: LifecycleStatus,

        /* Where? */
        nodes: Vec<String>,
        constraints: StartConstraints,
        scaling: ScalingPolicy,

        /* How? */
        resources: Resources,
        spec: Spec,
    }

    impl StoredGroup {
        pub fn get_status(&self) -> &LifecycleStatus {
            &self.status
        }
        pub fn get_nodes(&self) -> &Vec<String> {
            &self.nodes
        }
        pub fn get_nodes_mut(&mut self) -> &mut Vec<String> {
            &mut self.nodes
        }
        pub fn get_constraints(&self) -> &StartConstraints {
            &self.constraints
        }
        pub fn get_scaling(&self) -> &ScalingPolicy {
            &self.scaling
        }
        pub fn get_resources(&self) -> &Resources {
            &self.resources
        }
        pub fn get_spec(&self) -> &Spec {
            &self.spec
        }
    }

    impl LoadFromTomlFile for StoredGroup {}
    impl SaveToTomlFile for StoredGroup {}
}
