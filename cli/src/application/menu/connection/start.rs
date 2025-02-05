use std::fmt::Display;

use simplelog::debug;

use crate::application::{
    menu::{MenuResult, MenuUtils},
    network::EstablishedConnection,
    profile::{Profile, Profiles},
};

use super::{
    cloudlet::{
        create_cloudlet::CreateCloudletMenu, get_cloudlet::GetCloudletMenu,
        get_cloudlets::GetCloudletsMenu,
    },
    deployment::{
        create_deployment::CreateDeploymentMenu, get_deployment::GetDeploymentMenu,
        get_deployments::GetDeploymentsMenu,
    },
    general::{get_versions::GetVersionsMenu, request_stop::RequestStopMenu},
    resource::{delete_resource::DeleteResourceMenu, set_resource_status::SetResourceStatusMenu},
    unit::{get_unit::GetUnitMenu, get_units::GetUnitsMenu},
    user::transfer_user::TransferUserMenu,
};

enum Action {
    // Resource Management
    SetResourceStatus,
    DeleteResource,

    // Cloudlet Management
    CreateCloudlet,
    GetCloudlet,
    GetCloudlets,

    // Deployment Management
    CreateDeployment,
    GetDeployment,
    GetDeployments,

    // Unit Management
    GetUnit,
    GetUnits,

    // User Management
    TransferUser,

    // General
    RequestStop,
    GetVersions,

    // Connection
    DisconnectFromController,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::SetResourceStatus => write!(f, "Set status of a certain Resource"),
            Action::DeleteResource => write!(f, "Delete Resource"),

            Action::CreateCloudlet => write!(f, "Create Cloudlet"),
            Action::GetCloudlet => write!(f, "Get information about a certain Cloudlet"),
            Action::GetCloudlets => write!(f, "Get all Cloudlets"),

            Action::CreateDeployment => write!(f, "Create Deployment"),
            Action::GetDeployment => write!(f, "Get information about a certain Deployment"),
            Action::GetDeployments => write!(f, "Get all Deployments"),

            Action::GetUnit => write!(f, "Get information about a certain Unit"),
            Action::GetUnits => write!(f, "Get all Units"),

            Action::TransferUser => write!(f, "Transfer a user to a different Unit"),

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
                MenuResult::Exit => return MenuResult::Success,
                MenuResult::Error(error) => return MenuResult::Error(error),
                _ => {}
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
                Action::TransferUser,
                Action::SetResourceStatus,
                Action::DeleteResource,
                Action::CreateCloudlet,
                Action::CreateDeployment,
                Action::GetCloudlet,
                Action::GetDeployment,
                Action::GetUnit,
                Action::GetDeployments,
                Action::GetCloudlets,
                Action::GetUnits,
                Action::GetVersions,
                Action::DisconnectFromController,
            ],
        ) {
            Ok(selection) => match selection {
                Action::SetResourceStatus => {
                    SetResourceStatusMenu::show(profile, connection, profiles).await
                }
                Action::DeleteResource => {
                    DeleteResourceMenu::show(profile, connection, profiles).await
                }
                Action::CreateCloudlet => {
                    CreateCloudletMenu::show(profile, connection, profiles).await
                }
                Action::GetCloudlet => GetCloudletMenu::show(profile, connection, profiles).await,
                Action::GetCloudlets => GetCloudletsMenu::show(profile, connection, profiles).await,
                Action::CreateDeployment => {
                    CreateDeploymentMenu::show(profile, connection, profiles).await
                }
                Action::GetDeployment => {
                    GetDeploymentMenu::show(profile, connection, profiles).await
                }
                Action::GetDeployments => {
                    GetDeploymentsMenu::show(profile, connection, profiles).await
                }
                Action::GetUnit => GetUnitMenu::show(profile, connection, profiles).await,
                Action::GetUnits => GetUnitsMenu::show(profile, connection, profiles).await,
                Action::TransferUser => TransferUserMenu::show(profile, connection, profiles).await,
                Action::RequestStop => RequestStopMenu::show(profile, connection, profiles).await,
                Action::GetVersions => GetVersionsMenu::show(profile, connection, profiles).await,
                Action::DisconnectFromController => MenuResult::Exit,
            },
            Err(error) => {
                debug!("{}", error);
                MenuResult::Exit
            }
        }
    }
}
