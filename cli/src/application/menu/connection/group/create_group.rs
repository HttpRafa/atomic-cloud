use std::{fmt::Display, str::FromStr, vec};

use anyhow::{anyhow, Result};
use inquire::{
    validator::{Validation, ValueRequiredValidator},
    InquireError, MultiSelect, Text,
};
use loading::Loading;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::{
            common::KeyValue,
            manage::{
                group::{self, Constraints, Scaling},
                server::{DiskRetention, Fallback, Resources, Spec},
            },
        },
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

use std::fmt::Write as _;

pub struct CreateGroupMenu;

struct Data {
    groups: Vec<String>,
    nodes: Vec<String>,
}

impl CreateGroupMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving all existing groups from the controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok(data) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();

                match Self::collect_group(&data) {
                    Ok(group) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Creating group \"{}\" on the controller \"{}\"...",
                            group.name, profile.name
                        ));

                        match connection.client.create_group(group).await {
                            Ok(()) => {
                                progress.success("Group created successfully 👍. Remember to set the group to active, or the controller won't start servers.");
                                progress.end();
                                MenuResult::Success
                            }
                            Err(error) => {
                                progress.fail(format!("{error}"));
                                progress.end();
                                MenuResult::Failed(error)
                            }
                        }
                    }
                    Err(error) => MenuUtils::handle_error(error),
                }
            }
            Err(error) => {
                progress.fail(format!("{error}"));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<Data> {
        let groups = connection.client.get_groups().await?;
        let nodes = connection.client.get_nodes().await?;
        Ok(Data { groups, nodes })
    }

    fn collect_group(data: &Data) -> Result<group::Item, InquireError> {
        let name = Self::get_group_name(data.groups.clone())?;
        let nodes = Self::get_nodes(data.nodes.clone())?;
        let constraints = Self::collect_constraints()?;
        let scaling = Self::collect_scaling()?;
        let resources = Self::collect_resources()?;
        let spec = Self::collect_specification()?;

        Ok(group::Item {
            name,
            nodes,
            constraints: Some(constraints),
            scaling: Some(scaling),
            resources: Some(resources),
            spec: Some(spec),
        })
    }

    fn get_group_name(used_names: Vec<String>) -> Result<String, InquireError> {
        Text::new("What would you like to name this cloudGroup?")
            .with_help_message("Examples: lobby, mode-xyz")
            .with_validator(ValueRequiredValidator::default())
            .with_validator(move |name: &str| {
                if used_names.contains(&name.to_string()) {
                    Ok(Validation::Invalid(
                        "A cloudGroup with this name already exists".into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt()
    }

    fn get_nodes(nodes: Vec<String>) -> Result<Vec<String>, InquireError> {
        MultiSelect::new("What nodes should this cloudGroup use?", nodes).prompt()
    }

    fn collect_constraints() -> Result<Constraints, InquireError> {
        let min = MenuUtils::parsed_value(
            "What is the minimum number of servers that should always be online?",
            "Example: 1",
            "Please enter a valid number",
        )?;
        let max = MenuUtils::parsed_value(
            "What is the maximum number of servers that should always be online?",
            "Example: 10",
            "Please enter a valid number",
        )?;
        let prio = MenuUtils::parsed_value("How important is this cloudGroup compared to others? (This refers to one tick of the controller)", "Example: 0", "Please enter a valid number")?;

        Ok(Constraints { min, max, prio })
    }

    fn collect_scaling() -> Result<Scaling, InquireError> {
        let start_threshold = MenuUtils::parsed_value::<f32>("At what percentage (0-100) of the max player count should the controller start a new server?", "Example: 50", "Please enter a valid number")? / 100.0;
        let stop_empty =
            MenuUtils::confirm("Should the controller stop servers that are empty for too long?")?;

        Ok(Scaling {
            start_threshold,
            stop_empty,
        })
    }

    fn collect_resources() -> Result<Resources, InquireError> {
        let memory = MenuUtils::parsed_value(
            "How much memory should each server have?",
            "Example: 2048",
            "Please enter a valid number",
        )?;
        let swap = MenuUtils::parsed_value(
            "How much swap space should each server have?",
            "Example: 256",
            "Please enter a valid number",
        )?;
        let cpu = MenuUtils::parsed_value(
            "How much CPU power should each server have? (100 = one core)",
            "Example: 500",
            "Please enter a valid number",
        )?;
        let io = MenuUtils::parsed_value(
            "How many I/O operations should each server be allowed to perform?",
            "Example: 500",
            "Please enter a valid number",
        )?;
        let disk = MenuUtils::parsed_value(
            "How much disk space should each server use?",
            "Example: 2048",
            "Please enter a valid number",
        )?;
        let ports = MenuUtils::parsed_value(
            "How many addresses/ports should each server have?",
            "Example: 5",
            "Please enter a valid number",
        )?;

        Ok(Resources {
            memory,
            swap,
            cpu,
            io,
            disk,
            ports,
        })
    }

    fn collect_specification() -> Result<Spec, InquireError> {
        let img = MenuUtils::text(
            "Which image should the server use?",
            "Example: ubuntu:latest",
        )?;
        let max_players = MenuUtils::parsed_value(
            "What is the maximum number of players per server?",
            "Example: 20",
            "Please enter a valid number",
        )?;
        let settings = MenuUtils::parsed_value::<KeyValueList>(
            "What settings should the controller pass to the plugin when starting a server?",
            "Format: key=value,key=value,key=value,... (or leave it empty for none)",
            "Please check your syntax. Something seems wrong.",
        )?
        .key_values;
        let env = MenuUtils::parsed_value::<KeyValueList>(
            "What environment variables should the controller pass to the plugin when starting a server?",
            "Format: key=value,key=value,key=value,... (or leave it empty for none)",
            "Please check your syntax something is wrong",
        )?
        .key_values;
        let retention = MenuUtils::select_no_help(
            "Should the server's disk be retained after the server stops?",
            vec![DiskRetention::Temporary, DiskRetention::Permanent],
        )?;
        let fallback = Self::collect_fallback()?;

        Ok(Spec {
            img,
            max_players,
            settings,
            env,
            retention: Some(retention as i32),
            fallback,
        })
    }

    fn collect_fallback() -> Result<Option<Fallback>, InquireError> {
        let enabled =
            MenuUtils::confirm("Should the controller treat these servers as fallback servers?")?;
        let prio = MenuUtils::parsed_value(
            "What is the priority of this fallback cloudGroup?",
            "Example: 0",
            "Please enter a valid number",
        )?;

        if enabled {
            Ok(Some(Fallback { prio }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
struct KeyValueList {
    key_values: Vec<KeyValue>,
}

impl FromStr for KeyValueList {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut result = Vec::new();
        if string.is_empty() {
            return Ok(KeyValueList { key_values: result });
        }
        for pair in string.split(',') {
            let mut parts = pair.split('=');
            let key = parts
                .next()
                .ok_or_else(|| anyhow!("No key found in pair '{}'", pair))?;
            let value = parts
                .next()
                .ok_or_else(|| anyhow!("No value found for key '{}' in pair '{}'", key, pair))?;
            result.push(KeyValue {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
        Ok(KeyValueList { key_values: result })
    }
}

impl Display for KeyValueList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for pair in &self.key_values {
            write!(&mut result, "{}={},", pair.key, pair.value).expect("Failed to write to string");
        }
        write!(f, "{result}")
    }
}
