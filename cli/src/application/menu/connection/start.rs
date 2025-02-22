use std::fmt::Display;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

use super::{
    general::{get_versions::GetVersionsMenu, request_stop::RequestStopMenu},
    group::{create_group::CreateGroupMenu, get_group::GetGroupMenu, get_groups::GetGroupsMenu},
    node::{create_node::CreateNodeMenu, get_node::GetNodeMenu, get_nodes::GetNodesMenu},
    resource::{delete_resource::DeleteResourceMenu, set_resource::SetResourceMenu},
    server::{get_server::GetServerMenu, get_servers::GetServersMenu},
    user::transfer_users::TransferUsersMenu,
};

enum Action {
    // Resource operations
    SetResource,
    DeleteResource,

    // Node operations
    CreateNode,
    GetNode,
    GetNodes,

    // Group operations
    CreateGroup,
    GetGroup,
    GetGroups,

    // Server operations
    GetServer,
    GetServers,

    // Transfer operations
    TransferUsers,

    // General
    RequestStop,
    GetVersions,

    // Connection
    DisconnectFromController,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::SetResource => write!(f, "Set status of a certain Resource"),
            Action::DeleteResource => write!(f, "Delete Resource"),

            Action::CreateNode => write!(f, "Create Node"),
            Action::GetNode => write!(f, "Get information about a certain Node"),
            Action::GetNodes => write!(f, "Get all Nodes"),

            Action::CreateGroup => write!(f, "Create Group"),
            Action::GetGroup => write!(f, "Get information about a certain Group"),
            Action::GetGroups => write!(f, "Get all Groups"),

            Action::GetServer => write!(f, "Get information about a certain Server"),
            Action::GetServers => write!(f, "Get all Servers"),

            Action::TransferUsers => write!(f, "Transfer a users to a different Server"),

            Action::RequestStop => write!(f, "Request stop of Controller"),
            Action::GetVersions => write!(f, "Get versions"),

            Action::DisconnectFromController => write!(f, "Disconnect from Controller"),
        }
    }
}

pub struct ConnectionStartMenu;

impl ConnectionStartMenu {
    pub async fn show(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        profiles: &mut Profiles,
    ) -> MenuResult {
        loop {
            match Self::show_internal(profile, connection, profiles).await {
                MenuResult::Aborted | MenuResult::Exit => return MenuResult::Success,
                MenuResult::Failed(error) => return MenuResult::Failed(error),
                MenuResult::Success => {}
            }
        }
    }

    async fn show_internal(
        profile: &mut Profile,
        connection: &mut EstablishedConnection,
        profiles: &mut Profiles,
    ) -> MenuResult {
        match MenuUtils::select_no_help(
            "What do you want to do?",
            vec![
                Action::RequestStop,
                Action::TransferUsers,
                Action::SetResource,
                Action::DeleteResource,
                Action::CreateNode,
                Action::CreateGroup,
                Action::GetNode,
                Action::GetGroup,
                Action::GetServer,
                Action::GetNodes,
                Action::GetGroups,
                Action::GetServers,
                Action::GetVersions,
                Action::DisconnectFromController,
            ],
        ) {
            Ok(selection) => match selection {
                Action::SetResource => SetResourceMenu::show(profile, connection, profiles).await,
                Action::DeleteResource => {
                    DeleteResourceMenu::show(profile, connection, profiles).await
                }
                Action::CreateNode => CreateNodeMenu::show(profile, connection, profiles).await,
                Action::GetNode => GetNodeMenu::show(profile, connection, profiles).await,
                Action::GetNodes => GetNodesMenu::show(profile, connection, profiles).await,
                Action::CreateGroup => CreateGroupMenu::show(profile, connection, profiles).await,
                Action::GetGroup => GetGroupMenu::show(profile, connection, profiles).await,
                Action::GetGroups => GetGroupsMenu::show(profile, connection, profiles).await,
                Action::GetServer => GetServerMenu::show(profile, connection, profiles).await,
                Action::GetServers => GetServersMenu::show(profile, connection, profiles).await,
                Action::TransferUsers => {
                    TransferUsersMenu::show(profile, connection, profiles).await
                }
                Action::RequestStop => RequestStopMenu::show(profile, connection, profiles).await,
                Action::GetVersions => GetVersionsMenu::show(profile, connection, profiles).await,
                Action::DisconnectFromController => MenuResult::Exit,
            },
            Err(error) => MenuUtils::handle_error(error),
        }
    }
}
