use std::{fmt::Display, str::FromStr, vec};

use anyhow::{anyhow, Result};
use inquire::{
    validator::{Validation, ValueRequiredValidator},
    MultiSelect, Text,
};
use loading::Loading;
use simplelog::debug;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::{
            common::KeyValue,
            deployment_management::{
                deployment_value::{Constraints, Scaling},
                DeploymentValue,
            },
            unit_management::{
                unit_spec::{Fallback, Retention},
                UnitResources, UnitSpec,
            },
        },
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

pub struct CreateDeploymentMenu;

struct Data {
    deployments: Vec<String>,
    cloudlets: Vec<String>,
}

impl CreateDeploymentMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving all existing deployments from the controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok(data) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();

                match Self::collect_deployment(&data) {
                    Ok(deployment) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Creating deployment \"{}\" on the controller \"{}\"...",
                            deployment.name, profile.name
                        ));

                        match connection.client.create_deployment(deployment).await {
                            Ok(_) => {
                                progress.success("Deployment created successfully 👍. Remember to set the deployment to active, or the controller won't start units.");
                                progress.end();
                                MenuResult::Success
                            }
                            Err(error) => {
                                progress.fail(format!("{}", error));
                                progress.end();
                                MenuResult::Failed
                            }
                        }
                    }
                    Err(error) => {
                        debug!("{}", error);
                        MenuResult::Failed
                    }
                }
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<Data> {
        let deployments = connection.client.get_deployments().await?;
        let cloudlets = connection.client.get_cloudlets().await?;
        Ok(Data {
            deployments,
            cloudlets,
        })
    }

    fn collect_deployment(data: &Data) -> Result<DeploymentValue> {
        let name = Self::get_deployment_name(data.deployments.clone())?;
        let cloudlets = Self::get_cloudlets(data.cloudlets.clone())?;
        let constraints = Self::collect_constraints()?;
        let scaling = Self::collect_scaling()?;
        let resources = Self::collect_resources()?;
        let spec = Self::collect_specification()?;

        Ok(DeploymentValue {
            name,
            cloudlets,
            constraints: Some(constraints),
            scaling: Some(scaling),
            resources: Some(resources),
            spec: Some(spec),
        })
    }

    fn get_deployment_name(used_names: Vec<String>) -> Result<String> {
        Text::new("What would you like to name this deployment?")
            .with_help_message("Examples: lobby, mode-xyz")
            .with_validator(ValueRequiredValidator::default())
            .with_validator(move |name: &str| {
                if used_names.contains(&name.to_string()) {
                    Ok(Validation::Invalid(
                        "A deployment with this name already exists".into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt()
            .map_err(|error| error.into())
    }

    fn get_cloudlets(cloudlets: Vec<String>) -> Result<Vec<String>> {
        MultiSelect::new("What cloudlets should this deployment use?", cloudlets)
            .prompt()
            .map_err(|error| error.into())
    }

    fn collect_constraints() -> Result<Constraints> {
        let minimum = MenuUtils::parsed_value(
            "What is the minimum number of units that should always be online?",
            "Example: 1",
            "Please enter a valid number",
        )?;
        let maximum = MenuUtils::parsed_value(
            "What is the maximum number of units that should always be online?",
            "Example: 10",
            "Please enter a valid number",
        )?;
        let priority = MenuUtils::parsed_value("How important is this deployment compared to others? (This refers to one tick of the controller)", "Example: 0", "Please enter a valid number")?;

        Ok(Constraints {
            minimum,
            maximum,
            priority,
        })
    }

    fn collect_scaling() -> Result<Scaling> {
        let max_players = MenuUtils::parsed_value(
            "What is the maximum number of players per unit?",
            "Example: 20",
            "Please enter a valid number",
        )?;
        let start_threshold = MenuUtils::parsed_value::<f32>("At what percentage (0-100) of the max player count should the controller start a new unit?", "Example: 50", "Please enter a valid number")? / 100.0;
        let stop_empty_units =
            MenuUtils::confirm("Should the controller stop units that are empty for too long?")?;

        Ok(Scaling {
            max_players,
            start_threshold,
            stop_empty_units,
        })
    }

    fn collect_resources() -> Result<UnitResources> {
        let memory = MenuUtils::parsed_value(
            "How much memory should each unit have?",
            "Example: 2048",
            "Please enter a valid number",
        )?;
        let swap = MenuUtils::parsed_value(
            "How much swap space should each unit have?",
            "Example: 256",
            "Please enter a valid number",
        )?;
        let cpu = MenuUtils::parsed_value(
            "How much CPU power should each unit have? (100 = one core)",
            "Example: 500",
            "Please enter a valid number",
        )?;
        let io = MenuUtils::parsed_value(
            "How many I/O operations should each unit be allowed to perform?",
            "Example: 500",
            "Please enter a valid number",
        )?;
        let disk = MenuUtils::parsed_value(
            "How much disk space should each unit use?",
            "Example: 2048",
            "Please enter a valid number",
        )?;
        let addresses = MenuUtils::parsed_value(
            "How many addresses/ports should each unit have?",
            "Example: 5",
            "Please enter a valid number",
        )?;

        Ok(UnitResources {
            memory,
            swap,
            cpu,
            io,
            disk,
            addresses,
        })
    }

    fn collect_specification() -> Result<UnitSpec> {
        let image = MenuUtils::text("Which image should the unit use?", "Example: ubuntu:latest")?;
        let settings = MenuUtils::parsed_value::<KeyValueList>(
            "What settings should the controller pass to the driver when starting a unit?",
            "Format: key=value,key=value,key=value,...",
            "Please check your syntax. Something seems wrong.",
        )?
        .key_values;
        let environment = MenuUtils::parsed_value::<KeyValueList>(
            "What environment variables should the controller pass to the driver when starting a unit?",
            "Format: key=value,key=value,key=value,...",
            "Please check your syntax something is wrong",
        )?
        .key_values;
        let disk_retention = MenuUtils::select_no_help(
            "Should the unit's disk be retained after the unit stops?",
            vec![Retention::Temporary, Retention::Permanent],
        )?;
        let fallback = Self::collect_fallback()?;

        Ok(UnitSpec {
            image,
            settings,
            environment,
            disk_retention: Some(disk_retention as i32),
            fallback: Some(fallback),
        })
    }

    fn collect_fallback() -> Result<Fallback> {
        let enabled =
            MenuUtils::confirm("Should the controller treat these units as fallback units?")?;
        let priority = MenuUtils::parsed_value(
            "What is the priority of this fallback deployment?",
            "Example: 0",
            "Please enter a valid number",
        )?;

        Ok(Fallback { enabled, priority })
    }
}

#[derive(Clone)]
struct KeyValueList {
    key_values: Vec<KeyValue>,
}

impl FromStr for KeyValueList {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Vec::new();
        for pair in s.split(',') {
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
            result.push_str(&format!("{}={},", pair.key, pair.value));
        }
        write!(f, "{}", result)
    }
}
