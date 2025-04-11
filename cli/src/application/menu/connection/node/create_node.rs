use anyhow::Result;
use inquire::{
    validator::{Validation, ValueRequiredValidator},
    InquireError, Text,
};
use loading::Loading;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::manage::node::{self, Capabilities},
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

pub struct CreateNodeMenu;

struct Data {
    nodes: Vec<String>,
    plugins: Vec<String>,
}

impl CreateNodeMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving all existing nodes from the controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok(data) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();

                match Self::collect_node(&data) {
                    Ok(node) => {
                        let progress = Loading::default();
                        progress.text(format!(
                            "Creating node \"{}\" on the controller \"{}\"...",
                            node.name, profile.name
                        ));

                        match connection.client.create_node(node).await {
                            Ok(()) => {
                                progress.success("Node created successfully 👍. Remember to set the node to active, or the controller won't start servers.");
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
        let nodes = connection.client.get_nodes().await?;
        let plugins = connection.client.get_plugins().await?;
        Ok(Data { nodes, plugins })
    }

    fn collect_node(data: &Data) -> Result<node::Item, InquireError> {
        let name = Self::get_node_name(data.nodes.clone())?;
        let plugin = MenuUtils::select("Which plugin should the controller use to communicate with the backend of this node?", "This is essential for the controller to know how to communicate with the backend of this node. For example, is it a Pterodactyl node or a simple Docker host?", data.plugins.clone())?;
        let capabilities = Self::collect_capabilities()?;
        let ctrl_addr = MenuUtils::parsed_value(
            "What is the hostname or address where the server can reach the controller once started?",
            "Example: https://cloud.your-network.net",
            "Please enter a valid URL",
        )?;

        Ok(node::Item {
            name,
            plugin,
            ctrl_addr,
            capabilities: Some(capabilities),
        })
    }

    fn collect_capabilities() -> Result<Capabilities, InquireError> {
        let child = Self::get_child_node()?;
        let memory = Self::get_memory_limit()?;
        let max = Self::get_servers_limit()?;

        Ok(Capabilities { memory, max, child })
    }

    fn get_node_name(used_names: Vec<String>) -> Result<String, InquireError> {
        Text::new("What would you like to name this node?")
            .with_help_message("Examples: hetzner-01, home-01, local-01")
            .with_validator(ValueRequiredValidator::default())
            .with_validator(move |name: &str| {
                if used_names.contains(&name.to_string()) {
                    Ok(Validation::Invalid(
                        "A node with this name already exists".into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt()
    }

    fn get_memory_limit() -> Result<Option<u32>, InquireError> {
        if MenuUtils::confirm(
            "Would you like to limit the amount of memory the controller can use on this node?",
        )? {
            Ok(Some(MenuUtils::parsed_value(
                "How much memory should the controller be allowed to use on this node?",
                "Example: 1024",
                "Please enter a valid number",
            )?))
        } else {
            Ok(None)
        }
    }

    fn get_servers_limit() -> Result<Option<u32>, InquireError> {
        if MenuUtils::confirm(
            "Would you like to limit the number of servers the controller can start on this node?",
        )? {
            Ok(Some(MenuUtils::parsed_value(
                "How many servers should the controller be allowed to start on this node?",
                "Example: 15",
                "Please enter a valid number",
            )?))
        } else {
            Ok(None)
        }
    }

    fn get_child_node() -> Result<Option<String>, InquireError> {
        if MenuUtils::confirm("Does the specified plugin need additional information to determine which node it should use in the backend? This is required when a plugin manages multiple nodes.")? {
            Ok(Some(Text::new("What is the name of the child node the controller should use?")
                .with_help_message("Example: node0.gameservers.my-pelican.net")
                .with_validator(ValueRequiredValidator::default())
                .prompt()?))
        } else { Ok(None) }
    }
}
