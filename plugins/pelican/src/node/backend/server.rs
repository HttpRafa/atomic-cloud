use common::name::TimedName;
use data::{
    BCServer, BCServerAllocation, BKeyValue, BResources, BServer, BServerEgg, BServerFeatureLimits,
    BSignal, BUpdateBuild, PanelState,
};

use crate::generated::{exports::plugin::system::bridge::Server, plugin::system::http::Method};

use super::{allocation::data::BAllocation, Backend, Endpoint};

pub mod data;

impl Backend {
    pub fn update_settings(
        &self,
        backend_server: &BServer,
        server: &Server,
        primary_allocation: u32,
        environment: &[(String, String)],
    ) {
        // Update the environment variables
        for (key, value) in environment {
            self.update_variable(&backend_server.identifier, key, value);
        }
        self.update_build_configuration(
            backend_server.id,
            &BUpdateBuild {
                allocation: primary_allocation,
                memory: server.allocation.resources.memory,
                swap: server.allocation.resources.swap,
                disk: server.allocation.resources.disk,
                io: server.allocation.resources.io,
                cpu: server.allocation.resources.cpu,
                threads: None,
                feature_limits: BServerFeatureLimits {
                    databases: 0,
                    backups: 0,
                },
            },
        );
    }

    pub fn update_variable(&self, identifier: &str, key: &str, value: &str) -> bool {
        let value = serde_json::to_vec(&BKeyValue {
            key: key.to_string(),
            value: value.to_string(),
        })
        .ok();
        self.send_to_api(
            Method::Put,
            &Endpoint::Client,
            &format!("servers/{}/startup/variable", &identifier),
            200,
            value.as_deref(),
            None,
        )
    }

    pub fn update_build_configuration(&self, id: u32, update_build: &BUpdateBuild) -> bool {
        let value = serde_json::to_vec(&update_build).ok();
        self.send_to_api(
            Method::Patch,
            &Endpoint::Application,
            &format!("servers/{}/build", &id),
            200,
            value.as_deref(),
            None,
        )
    }

    pub fn get_server_state(&self, identifier: &str) -> Option<PanelState> {
        self.get_server_resources(identifier).map(|resources| {
            match resources.current_state.as_str() {
                "running" => PanelState::Running,
                "stopping" => PanelState::Stopping,
                "offline" => PanelState::Offline,
                _ => PanelState::Starting,
            }
        })
    }

    fn get_server_resources(&self, identifier: &str) -> Option<BResources> {
        self.get_object_from_api::<(), BResources>(
            &Endpoint::Client,
            &format!("servers/{}/resources", &identifier),
            &(),
        )
        .map(|data| data.attributes)
    }

    pub fn start_server(&self, identifier: &str) -> bool {
        self.change_power_state(identifier, "start")
    }

    pub fn restart_server(&self, identifier: &str) -> bool {
        self.change_power_state(identifier, "restart")
    }

    pub fn stop_server(&self, identifier: &str) -> bool {
        self.change_power_state(identifier, "stop")
    }

    pub fn kill_server(&self, identifier: &str) -> bool {
        self.change_power_state(identifier, "kill")
    }

    fn change_power_state(&self, identifier: &str, state: &str) -> bool {
        let state = serde_json::to_vec(&BSignal {
            signal: state.to_string(),
        })
        .ok();
        self.send_to_api(
            Method::Post,
            &Endpoint::Client,
            &format!("servers/{}/power", &identifier),
            204,
            state.as_deref(),
            None,
        )
    }

    pub fn delete_server(&self, server: u32) -> bool {
        self.delete_in_api(&Endpoint::Application, &format!("servers/{server}"))
    }

    pub fn create_server(
        &self,
        name: &TimedName,
        server: &Server,
        allocations: &[BAllocation],
        environment: &[(String, String)],
        egg: &BServerEgg,
        features: &BServerFeatureLimits,
    ) -> Option<BServer> {
        let backend_server = BCServer {
            name: name.get_name_cloned(),
            node: self.node_id,
            user: self.user_id,
            egg: egg.id,
            docker_image: server.allocation.spec.image.clone(),
            startup: egg.startup.clone(),
            environment: environment.iter().cloned().collect(),
            limits: server.allocation.resources.into(),
            feature_limits: features.clone(),
            allocation: BCServerAllocation::from(allocations),
            start_on_completion: true,
        };
        self.post_object_to_api::<BCServer, BServer>(
            &Endpoint::Application,
            "servers",
            &backend_server,
        )
        .map(|data| data.attributes)
    }

    pub fn get_server_by_name(&self, name: &TimedName) -> Option<BServer> {
        self.api_find_on_pages::<BServer>(
            Method::Get,
            &Endpoint::Application,
            "servers",
            |object| {
                object
                    .data
                    .iter()
                    .find(|server| server.attributes.name == name.get_name())
                    .map(|server| server.attributes.clone())
            },
        )
    }
}
