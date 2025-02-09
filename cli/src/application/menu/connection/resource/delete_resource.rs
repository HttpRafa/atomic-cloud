use anyhow::Result;
use inquire::InquireError;
use loading::Loading;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::manage::{
            resource::{Category, DelReq},
            server,
        },
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

pub struct DeleteResourceMenu;

// TODO: Maybe dont request everything at once, but only what is needed
struct Data {
    nodes: Vec<String>,
    groups: Vec<String>,
    servers: Vec<server::Short>,
}

impl DeleteResourceMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        _profiles: &mut Profiles,
    ) -> MenuResult {
        let progress = Loading::default();
        progress.text(format!(
            "Retrieving all required data from the controller \"{}\"...",
            profile.name
        ));

        match Self::get_required_data(connection).await {
            Ok(data) => {
                progress.success("Data retrieved successfully 👍");
                progress.end();

                match Self::collect_delete_resource(&data) {
                    Ok(request) => {
                        let progress = Loading::default();
                        progress.text("Deleting resource...");

                        match connection.client.delete_resource(request).await {
                            Ok(()) => {
                                progress.success("Resource deleted successfully 👍.");
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
        let groups = connection.client.get_groups().await?;
        let servers = connection.client.get_servers().await?;
        Ok(Data {
            nodes,
            groups,
            servers,
        })
    }

    fn collect_delete_resource(data: &Data) -> Result<DelReq, InquireError> {
        let category = MenuUtils::select_no_help(
            "What type of resource to you want to delete?",
            vec![Category::Node, Category::Group, Category::Server],
        )?;
        match category {
            Category::Node => {
                let node =
                    MenuUtils::select_no_help("Select the node to delete", data.nodes.clone())?;
                Ok(DelReq {
                    category: category as i32,
                    id: node,
                })
            }
            Category::Group => {
                let group =
                    MenuUtils::select_no_help("Select the group to delete", data.groups.clone())?;
                Ok(DelReq {
                    category: category as i32,
                    id: group,
                })
            }
            Category::Server => {
                let server =
                    MenuUtils::select_no_help("Select the server to delete", data.servers.clone())?;
                Ok(DelReq {
                    category: category as i32,
                    id: server.id,
                })
            }
        }
    }
}
