use anyhow::Result;
use inquire::{
    validator::{Validation, ValueRequiredValidator},
    Text,
};
use loading::Loading;
use simplelog::debug;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{proto::cloudlet_management::CloudletValue, EstablishedConnection},
    profile::{Profile, Profiles},
};

pub struct CreateCloudletMenu;

struct Data {
    cloudlets: Vec<String>,
    drivers: Vec<String>,
}

impl CreateCloudletMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving all existing cloudlets from the controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok(data) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();

                match Self::collect_cloudlet(&data) {
                    Ok(cloudlet) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Creating cloudlet \"{}\" on the controller \"{}\"...",
                            cloudlet.name, profile.name
                        ));

                        match connection.client.create_cloudlet(cloudlet).await {
                            Ok(_) => {
                                progress.success("Cloudlet created successfully 👍. Remember to set the cloudlet to active, or the controller won't start units.");
                                progress.end();
                                MenuResult::Success
                            }
                            Err(error) => {
                                progress.fail(format!(
                                    "An error occurred while creating the cloudlet: {}",
                                    error
                                ));
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
                progress.fail(format!(
                    "An error occurred while fetching the required data from the controller: {}",
                    error
                ));
                progress.end();
                MenuResult::Failed
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<Data> {
        let cloudlets = connection.client.get_cloudlets().await?;
        let drivers = connection.client.get_drivers().await?;
        Ok(Data { cloudlets, drivers })
    }

    fn collect_cloudlet(data: &Data) -> Result<CloudletValue> {
        let name = Self::get_cloudlet_name(data.cloudlets.clone())?;
        let driver = MenuUtils::select("Which driver should the controller use to communicate with the backend of this cloudlet?", "This is essential for the controller to know how to communicate with the backend of this cloudlet. For example, is it a Pterodactyl node or a simple Docker host?", data.drivers.to_vec())?;
        let child = Self::get_child_node()?;
        let memory = Self::get_memory_limit()?;
        let max_allocations = Self::get_allocations_limit()?;
        let controller_address = MenuUtils::parsed_value(
            "What is the hostname or address where the unit can reach the controller once started?",
            "Example: https://cloud.your-network.net",
            "Please enter a valid URL",
        )?;

        Ok(CloudletValue {
            name,
            driver,
            memory,
            max_allocations,
            child,
            controller_address,
        })
    }

    fn get_cloudlet_name(used_names: Vec<String>) -> Result<String> {
        Text::new("What would you like to name this cloudlet?")
            .with_help_message("Examples: hetzner-01, home-01, local-01")
            .with_validator(ValueRequiredValidator::default())
            .with_validator(move |name: &str| {
                if used_names.contains(&name.to_string()) {
                    Ok(Validation::Invalid(
                        "A cloudlet with this name already exists".into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt()
            .map_err(|error| error.into())
    }

    fn get_memory_limit() -> Result<Option<u32>> {
        match MenuUtils::confirm(
            "Would you like to limit the amount of memory the controller can use on this cloudlet?",
        )? {
            false => Ok(None),
            true => Ok(Some(MenuUtils::parsed_value(
                "How much memory should the controller be allowed to use on this cloudlet?",
                "Example: 1024",
                "Please enter a valid number",
            )?)),
        }
    }

    fn get_allocations_limit() -> Result<Option<u32>> {
        match MenuUtils::confirm("Would you like to limit the number of units the controller can start on this cloudlet?")?
        {
            false => Ok(None),
            true => Ok(Some(MenuUtils::parsed_value("How many units should the controller be allowed to start on this cloudlet?", "Example: 15", "Please enter a valid number")?))
        }
    }

    fn get_child_node() -> Result<Option<String>> {
        match MenuUtils::confirm("Does the specified driver need additional information to determine which node it should use in the backend? This is required when a driver manages multiple nodes.")? {
            false => Ok(None),
            true => {
                Ok(Some(Text::new("What is the name of the child node the controller should use?")
                    .with_help_message("Example: node0.gameservers.my-pterodactyl.net")
                    .with_validator(ValueRequiredValidator::default())
                    .prompt()?))
            }
        }
    }
}
