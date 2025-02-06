use anyhow::Result;
use inquire::InquireError;
use loading::Loading;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::{
        proto::manage::resource::{set_req, Category, SetReq},
        EstablishedConnection,
    },
    profile::{Profile, Profiles},
};

pub struct SetResourceMenu;

// TODO: Maybe dont request everything at once, but only what is needed
struct Data {
    nodes: Vec<String>,
    groups: Vec<String>,
}

impl SetResourceMenu {
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

                match Self::collect_set_resource_status_request(&data) {
                    Ok(request) => {
                        let progress = Loading::default();
                        progress.text("Changing resource...");

                        match connection.client.set_resource(request).await {
                            Ok(_) => {
                                progress.success("Resource changed successfully 👍.");
                                progress.end();
                                MenuResult::Success
                            }
                            Err(error) => {
                                progress.fail(format!("{}", error));
                                progress.end();
                                MenuResult::Failed(error)
                            }
                        }
                    }
                    Err(error) => MenuUtils::handle_error(error),
                }
            }
            Err(error) => {
                progress.fail(format!("{}", error));
                progress.end();
                MenuResult::Failed(error)
            }
        }
    }

    async fn get_required_data(connection: &mut EstablishedConnection) -> Result<Data> {
        let nodes = connection.client.get_nodes().await?;
        let groups = connection.client.get_groups().await?;
        Ok(Data { nodes, groups })
    }

    fn collect_set_resource_status_request(data: &Data) -> Result<SetReq, InquireError> {
        let status = MenuUtils::select_no_help(
            "What is the new status of this resource?",
            vec![set_req::Status::Active, set_req::Status::Inactive],
        )?;
        let category = MenuUtils::select_no_help(
            "What type of resource to you want to change?",
            vec![Category::Node, Category::Group, Category::Server],
        )?;
        match category {
            Category::Node => {
                let node =
                    MenuUtils::select_no_help("Select the node to change", data.nodes.clone())?;
                Ok(SetReq {
                    category: category as i32,
                    id: node,
                    status: status as i32,
                })
            }
            Category::Group => {
                let group =
                    MenuUtils::select_no_help("Select the group to change", data.groups.clone())?;
                Ok(SetReq {
                    category: category as i32,
                    id: group,
                    status: status as i32,
                })
            }
            Category::Server => Err(InquireError::OperationInterrupted),
        }
    }
}
